use ngx::ffi::{
    nginx_version, ngx_array_push, ngx_command_t, ngx_conf_t, ngx_http_core_module,
    ngx_http_handler_pt, ngx_http_module_t, ngx_http_phases_NGX_HTTP_ACCESS_PHASE,
    ngx_http_request_t, ngx_int_t, ngx_module_t, ngx_str_t, ngx_uint_t, NGX_CONF_TAKE1,
    NGX_HTTP_LOC_CONF, NGX_HTTP_MODULE, NGX_RS_HTTP_LOC_CONF_OFFSET, NGX_RS_MODULE_SIGNATURE,
};
use ngx::http::MergeConfigError;
use ngx::{core, core::Status, http, http::HTTPModule};
use ngx::{http_request_handler, ngx_null_command, ngx_string};
use sha2::{Digest, Sha256};
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
        *h = Some(bearer_auth_handler);
        core::Status::NGX_OK.into()
    }
}

#[derive(Debug, Default)]
struct ModuleConfig {
    hashed_token: Option<String>,
}

impl http::Merge for ModuleConfig {
    fn merge(&mut self, prev: &Self) -> Result<(), MergeConfigError> {
        if prev.hashed_token.is_some() {
            self.hashed_token.clone_from(&prev.hashed_token);
        }
        Ok(())
    }
}

#[no_mangle]
static mut ngx_bearer_auth: [ngx_command_t; 2] = [
    ngx_command_t {
        name: ngx_string!("bearer_auth"),
        type_: (NGX_HTTP_LOC_CONF | NGX_CONF_TAKE1) as ngx_uint_t,
        set: Some(ngx_bearer_auth_set_enable),
        conf: NGX_RS_HTTP_LOC_CONF_OFFSET,
        offset: 0,
        post: std::ptr::null_mut(),
    },
    ngx_null_command!(),
];

#[no_mangle]
static ngx_bearer_auth_module_ctx: ngx_http_module_t = ngx_http_module_t {
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
ngx::ngx_modules!(ngx_bearer_auth_module);

#[no_mangle]
#[used]
pub static mut ngx_bearer_auth_module: ngx_module_t = ngx_module_t {
    ctx_index: ngx_uint_t::MAX,
    index: ngx_uint_t::MAX,
    name: std::ptr::null_mut(),
    spare0: 0,
    spare1: 0,
    version: nginx_version as ngx_uint_t,
    signature: NGX_RS_MODULE_SIGNATURE.as_ptr() as *const c_char,

    ctx: &ngx_bearer_auth_module_ctx as *const _ as *mut _,
    commands: unsafe { &ngx_bearer_auth[0] as *const _ as *mut _ },
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

http_request_handler!(bearer_auth_handler, |request: &mut http::Request| {
    let co =
        unsafe { request.get_module_loc_conf::<ModuleConfig>(&*addr_of!(ngx_bearer_auth_module)) };
    let co = co.expect("module config is none");

    if co.hashed_token.is_some() {
        let recieved_token = request
            .headers_in_iterator()
            .find(|(k, v)| {
                k.eq_ignore_ascii_case("Authorization") && v.starts_with("Bearer ") && v.len() > 7
            })
            .map(|(_, v)| format!("{:x}", Sha256::digest(v[7..].as_bytes())));
        if recieved_token.is_some_and(|t| &t == co.hashed_token.as_ref().unwrap()) {
            core::Status::NGX_OK
        } else {
            http::HTTPStatus::FORBIDDEN.into()
        }
    } else {
        core::Status::NGX_OK
    }
});

#[no_mangle]
extern "C" fn ngx_bearer_auth_set_enable(
    cf: *mut ngx_conf_t,
    _cmd: *mut ngx_command_t,
    conf: *mut c_void,
) -> *mut c_char {
    unsafe {
        let conf = &mut *(conf as *mut ModuleConfig);
        let args = (*(*cf).args).elts as *mut ngx_str_t;

        let val = (*args.add(1)).to_str();

        // set default value optionally
        conf.hashed_token = None;

        if !val.is_empty() {
            conf.hashed_token = Some(val.to_string());
        }
    };

    std::ptr::null_mut()
}
