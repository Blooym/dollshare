#!/bin/sh
set -e

if [ -z "$1" ]
  then
    echo "Usage: dollhouse-share <file>"
    exit 1
fi

file=$(realpath $1)

curl -H "Authorization: Bearer <TOKEN_HERE>" https://<API_URL_HERE>/api/upload -F file="@$file" -H "Content-Type: multipart/form-data" | jq -r '.url' | wl-copy;
echo "Upload URL has been copied to clipboard"