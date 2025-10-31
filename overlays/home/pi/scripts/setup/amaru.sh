#!/bin/bash

set -euo pipefail

echo 'export PATH="$PATH:/home/pi/bin"' >> ~/.profile

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ARCHIVE="$SCRIPT_DIR/../bin/dbs.tar.gz"
if [ ! -f "$ARCHIVE" ]; then
  echo "Warning: Archive '$ARCHIVE' does not exist."
  exit 0
fi

set -- *.db
if [ -e "$1" ] && [ -z "${FORCE_UNPACK:-}" ]; then
  echo "Found existing .db files. Skipping unpack. (Use FORCE_UNPACK=1 to override.)"
  exit 0
fi

echo "Extracting $ARCHIVE..."
tar -xf "$ARCHIVE"