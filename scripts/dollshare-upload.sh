#!/bin/sh
# --------
# Make sure these are set.
DOLLSHARE_TOKEN=
DOLLSHARE_BASE_URL=
# --------
set -e

# Ensure env is set.
if [ -z "$DOLLSHARE_TOKEN" ]; then 
  echo "DOLLSHARE_TOKEN is not set."
  exit 1
fi
if [ -z "$DOLLSHARE_BASE_URL" ]; then 
  echo "DOLLSHARE_BASE_URL is not set."
  exit 1
fi

require_dependency() {
  if ! type $1 &> /dev/null; then
    echo "Missing dependency: $1 must be installed for this script to work"
    exit 1
  fi
}
require_dependency wl-copy
require_dependency curl

if [ -z "$1" ]
  then
    echo "Usage: dollshare-share <file>"
    exit 1
fi

file=$(realpath $1)
curl -H "Authorization: Bearer $DOLLSHARE_TOKEN" $DOLLSHARE_BASE_URL/upload -F file="@$file" -H "Content-Type: multipart/form-data" | jq -r '.url' | wl-copy;
echo "Upload URL has been copied to clipboard"