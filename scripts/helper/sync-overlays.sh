#!/usr/bin/env bash

set -euo pipefail

SSH_REMOTE="${SSH_REMOTE:-$1}"
SSH_OPTS="${SSH_OPTS:-}"
OVERLAYS_DIR="overlays"

if [[ -z "$SSH_REMOTE" ]]; then
  echo "Usage: $0 user@host"
  exit 2
fi

[[ -d "${OVERLAYS_DIR}" ]] || { echo "Error: '${OVERLAYS_DIR}' directory not found."; exit 1; }

echo "🔄 Uploading overlays/ → ${SSH_REMOTE}:/ ..."
rsync -rptl --progress --partial -e "ssh ${SSH_OPTS}" -r "${OVERLAYS_DIR}/home/pi/" "$SSH_REMOTE:./"
rsync -rlt --progress --partial -e "ssh ${SSH_OPTS}" --rsync-path="sudo rsync" "${OVERLAYS_DIR}/" "$SSH_REMOTE:/" --exclude="home/"
echo "✅ Upload complete."

echo "🔨 Configuring scripts on Pi..."
ssh $SSH_OPTS "$SSH_REMOTE" "
    find /home/pi/scripts -type f -name '*.sh' -exec chmod +x {} \;
    
    ln -sf /home/pi/scripts/updater.sh /home/pi/bin/updater.sh
    ln -sf /home/pi/scripts/activate-update.sh /home/pi/bin/activate-update.sh
"
echo "✅ Sync finished. Scripts configured."
