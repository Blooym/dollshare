#!/bin/sh
# --------
# Make sure these are set.
DOLLHOUSE_TOKEN=
DOLLHOUSE_BASE_URL=
# --------
set -e

# Ensure env is set.
if [ -z "$DOLLHOUSE_TOKEN" ]; then 
  echo "DOLLHOUSE_TOKEN is not set."
  exit 1
fi
if [ -z "$DOLLHOUSE_BASE_URL" ]; then 
  echo "DOLLHOUSE_BASE_URL is not set."
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
    echo "Usage: dollhouse-share <file>"
    exit 1
fi

file=$(realpath $1)
curl -H "Authorization: Bearer $DOLLHOUSE_TOKEN" $DOLLHOUSE_BASE_URL/api/upload -F file="@$file" -H "Content-Type: multipart/form-data" | jq -r '.url' | wl-copy;
echo "Upload URL has been copied to clipboard"