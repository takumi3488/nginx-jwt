# nginx-jwt

Nginx module providing JWT authentication and login page. Currently only `x86_64-unknown-linux-gnu` is supported.

## Usage

Download `libjwt.so` from Releases page and put it in `/etc/nginx/modules`.

Set the config file as follows, and start Nginx, the JWT will be required for pages with `/private` prefix. Redirect to `/login` if JWT is not sent or is invalid.

```
location = /login {
       alias /usr/share/nginx/html/login.html;
}

location /private {
       jwt YOUR_256_BIT_SECRET /login;
}
```
