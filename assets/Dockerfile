FROM nginx:1.27.0-bookworm

COPY ./libjwt.so /etc/nginx/modules/libjwt.so
COPY ./index.html /usr/share/nginx/html/index.html
COPY ./403.html /usr/share/nginx/html/403.html
COPY ./test.conf.template /etc/nginx/templates/test.conf.template
COPY ./entrypoint.sh /etc/nginx/entrypoint.sh

HEALTHCHECK --interval=3s --timeout=5s --retries=15 CMD curl -f http://localhost:80/public || exit 1
ENTRYPOINT [ "bash", "/etc/nginx/entrypoint.sh" ]
CMD ["nginx", "-c", "/etc/nginx/conf.d/test.conf"]
