use jsonwebtoken::{DecodingKey, Validation};
use ngx::ffi::{
    nginx_version, ngx_array_push, ngx_command_t, ngx_conf_t, ngx_http_core_module, ngx_http_handler_pt, ngx_http_module, ngx_http_module_t, ngx_http_phases_NGX_HTTP_ACCESS_PHASE, ngx_http_request_t, ngx_int_t, ngx_module_t, ngx_str_t, ngx_uint_t, NGX_CONF_TAKE2, NGX_HTTP_LOC_CONF, NGX_HTTP_MODULE, NGX_RS_HTTP_LOC_CONF_OFFSET, NGX_RS_MODULE_SIGNATURE
};
use ngx::http::MergeConfigError;
use ngx::{core, core::Status, http, http::HTTPModule};
use ngx::{http_request_handler, ngx_null_command, ngx_string};
use serde_json::Value;
use std::os::raw::{c_char, c_void};
use std::ptr::addr_of;

struct Module;

impl http::HTTPModule for Module {
    type MainConf = ();
    type SrvConf = ();
    type LocConf = ModuleConfig;

    unsafe extern "C" fn postconfiguration(cf: *mut ngx_conf_t) -> ngx_int_t {
        let cmcf = http::ngx_http_conf_get_module_main_conf(cf, &*addr_of!(ngx_http_core_module));

        let h = ngx_array_push(
            &mut (*cmcf).phases[ngx_http_phases_NGX_HTTP_ACCESS_PHASE as usize].handlers,
        ) as *mut ngx_http_handler_pt;
        if h.is_null() {
            return core::Status::NGX_ERROR.into();
        }
        *h = Some(jwt_handler);
        core::Status::NGX_OK.into()
    }
}

#[derive(Debug, Default)]
struct ModuleConfig {
    enabled: bool,
    secret: String,
    login_uri: String,
}

impl ModuleConfig {
    fn reset(&mut self) {
        self.enabled = false;
        self.secret.clear();
        self.login_uri.clear();
    }

    fn set(&mut self, secret: String, login_uri: String) {
        self.secret = secret;
        self.login_uri = login_uri;
        if self.secret.is_empty() || self.login_uri.is_empty() {
            self.enabled = false;
        } else {
            self.enabled = true;
        }
    }
}

impl http::Merge for ModuleConfig {
    fn merge(&mut self, prev: &Self) -> Result<(), MergeConfigError> {
        if prev.enabled {
            self.enabled = true;
        }

        if self.secret.is_empty() {
            self.secret = String::from(if !prev.secret.is_empty() {
                &prev.secret
            } else {
                ""
            });
        }

        if self.login_uri.is_empty() {
            self.login_uri = String::from(if !prev.login_uri.is_empty() {
                &prev.login_uri
            } else {
                ""
            });
        }

        if self.enabled && (self.secret.is_empty() || self.login_uri.is_empty()) {
            return Err(MergeConfigError::NoValue);
        }

        Ok(())
    }
}

#[no_mangle]
static mut ngx_jwt: [ngx_command_t; 2] = [
    ngx_command_t {
        name: ngx_string!("jwt"),
        type_: (NGX_HTTP_LOC_CONF | NGX_CONF_TAKE2) as ngx_uint_t,
        set: Some(ngx_jwt_set_args),
        conf: NGX_RS_HTTP_LOC_CONF_OFFSET,
        offset: 0,
        post: std::ptr::null_mut(),
    },
    ngx_null_command!(),
];

#[no_mangle]
static ngx_jwt_module_ctx: ngx_http_module_t = ngx_http_module_t {
    preconfiguration: Some(Module::preconfiguration),
    postconfiguration: Some(Module::postconfiguration),
    create_main_conf: Some(Module::create_main_conf),
    init_main_conf: Some(Module::init_main_conf),
    create_srv_conf: Some(Module::create_srv_conf),
    merge_srv_conf: Some(Module::merge_srv_conf),
    create_loc_conf: Some(Module::create_loc_conf),
    merge_loc_conf: Some(Module::merge_loc_conf),
};

#[cfg(feature = "export-modules")]
ngx::ngx_modules!(ngx_jwt_module);

#[no_mangle]
#[used]
pub static mut ngx_jwt_module: ngx_module_t = ngx_module_t {
    ctx_index: ngx_uint_t::MAX,
    index: ngx_uint_t::MAX,
    name: std::ptr::null_mut(),
    spare0: 0,
    spare1: 0,
    version: nginx_version as ngx_uint_t,
    signature: NGX_RS_MODULE_SIGNATURE.as_ptr() as *const c_char,

    ctx: &ngx_jwt_module_ctx as *const _ as *mut _,
    commands: unsafe { &ngx_jwt[0] as *const _ as *mut _ },
    type_: NGX_HTTP_MODULE as ngx_uint_t,

    init_master: None,
    init_module: None,
    init_process: None,
    init_thread: None,
    exit_thread: None,
    exit_process: None,
    exit_master: None,

    spare_hook0: 0,
    spare_hook1: 0,
    spare_hook2: 0,
    spare_hook3: 0,
    spare_hook4: 0,
    spare_hook5: 0,
    spare_hook6: 0,
    spare_hook7: 0,
};

http_request_handler!(jwt_handler, |request: &mut http::Request| {
    let co = unsafe { request.get_module_loc_conf::<ModuleConfig>(&*addr_of!(ngx_jwt_module)) };
    let co = co.expect("module config is none");

    if !co.enabled {
        println!("Module is not enabled");
        return core::Status::NGX_OK;
    }

    let is_valid = request
        .headers_in_iterator()
        .find(|(k, v)| k.eq_ignore_ascii_case("Cookie") && v.starts_with("token="))
        .map(|(_, v)| v.replace("token=", ""))
        .map(|v| {
            jsonwebtoken::decode::<Value>(
                &v,
                &DecodingKey::from_secret(co.secret.as_ref()),
                &Validation::default(),
            )
            .is_ok()
        })
        .unwrap_or(false);

    if !is_valid {
        let uri = format!(
            "{}?redirect_to={}",
            co.login_uri.as_str(),
            request.unparsed_uri().to_str().unwrap()
        );
        // request.set_status(http::HTTPStatus(303));
        // request.add_header_out("location", &location);
        return unsafe {
            request.subrequest(&uri, &*addr_of!(ngx_jwt_module))
        };
    }

    core::Status::NGX_OK
});

#[no_mangle]
extern "C" fn ngx_jwt_set_args(
    cf: *mut ngx_conf_t,
    _cmd: *mut ngx_command_t,
    conf: *mut c_void,
) -> *mut c_char {
    unsafe {
        let conf = &mut *(conf as *mut ModuleConfig);
        let args: *mut ngx_str_t = (*(*cf).args).elts as *mut ngx_str_t;
        let first_arg = (*args.add(1)).to_str();
        let second_arg = (*args.add(2)).to_str();
        if first_arg.is_empty() || second_arg.is_empty() {
            conf.reset();
            return "Invalid number of arguments".as_ptr() as *mut c_char;
        }
        conf.set(first_arg.to_string(), second_arg.to_string());
    };

    std::ptr::null_mut()
}
