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
require_dependency flameshot

screenshot_dir=/tmp/dollshare-flameshot
file="$screenshot_dir/$(date '+%h_%Y_%d_%I_%m_%S.png')";
mkdir -p $screenshot_dir
flameshot gui -r > $file; if [ ! -s $file ]; then
  exit 1;
fi

curl -H "Authorization: Bearer $DOLLSHARE_TOKEN" $DOLLSHARE_BASE_URL/upload -F file="@$file" -H "Content-Type: multipart/form-data" | jq -r '.url' | wl-copy;