#!/bin/bash -e

if [ -z "$JWT_SECRET" ]; then
  echo "JWT_SECRET is not set. Exiting."
  exit 1
fi
envsubst < /etc/nginx/templates/test.conf.template > /etc/nginx/conf.d/test.conf
cat /etc/nginx/conf.d/test.conf
eval "$@"
