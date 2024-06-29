# nginx-bearer-authentication

Nginx module for Bearer authentication. Currently only `x86_64-unknown-linux-gnu` is supported.

## Usage

Download `libbearerauth.so` from Releases page and put it in `/etc/nginx/modules`.

Set the config file as follows, and start Nginx, the Bearer token will be required for pages with `/private` prefix.

```
location /private {
       bearer_auth PUT_HASHED_TOKEN_HERE;
}
```
