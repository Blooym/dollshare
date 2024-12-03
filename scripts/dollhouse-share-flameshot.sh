#!/bin/sh
set -e

flameshot gui -r > /tmp/screenshot.png;if [ ! -s /tmp/screenshot.png ]; then
  exit 1
fi

curl -H "Authorization: Bearer <TOKEN_HERE>" https://<API_URL_HERE>/api/upload -F file="@/tmp/screenshot.png" -H "Content-Type: multipart/form-data" | jq -r '.url' | wl-copy;