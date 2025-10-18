#!/usr/bin/env sh
set -e

ARCHIVE=dbs.tar.gz
if [ ! -f "$ARCHIVE" ]; then
  echo "Error: Archive '$ARCHIVE' does not exist."
  exit 1
fi

tar -xf $ARCHIVE