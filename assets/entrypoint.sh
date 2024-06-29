#!/bin/bash -e

if [ -z "$BEARER_TOKEN" ]; then
  echo "BEARER_TOKEN is not set. Exiting."
  exit 1
fi
envsubst < /etc/nginx/templates/test.conf.template > /etc/nginx/conf.d/test.conf
eval "$@"
