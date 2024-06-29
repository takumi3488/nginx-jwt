# nginx-jwt

Nginx module providing JWT authentication and login page. Currently only `x86_64-unknown-linux-gnu` is supported.

## Usage

Download `libjwt.so` from Releases page and load it from config file of nginx.

Set the config file as follows, the JWT will be required for pages with `/private` prefix. Redirect to `/login` if JWT is not sent or is invalid. 

```
load_module /etc/nginx/modules/libjwt.so;

location = /login {
       alias /var/www/login.php;
}

location /private {
       jwt YOUR_256_BIT_SECRET /login;
}
```
