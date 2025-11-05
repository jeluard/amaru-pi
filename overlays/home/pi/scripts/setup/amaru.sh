#!/bin/bash

set -euo pipefail

echo 'export PATH="$PATH:/home/pi/bin"' >> ~/.profile

BIN_DIR="/home/pi/bin"
ARCHIVE="${BIN_DIR}/dbs.tar.gz"
if [ ! -f "${ARCHIVE}" ]; then
  echo "⚠️ Warning: Archive '${ARCHIVE}' does not exist."
  exit 0
fi

cd ${BIN_DIR}

set -- *.db # Sets all `*.db` file as positional parameters
if [ -e "$1" ] && [ -z "${FORCE_UNPACK:-}" ]; then
  echo "➡️ Found existing .db files. Skipping unpack. (Use FORCE_UNPACK=1 to override.)"
  exit 0
fi

echo "➡️ Extracting ${ARCHIVE}..."
tar -xf "${ARCHIVE}"