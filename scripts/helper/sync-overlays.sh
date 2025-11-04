#!/usr/bin/env bash

set -euo pipefail

HELPER_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="${HELPER_DIR}/../.."
OVERLAYS_DIR="${ROOT_DIR}/overlays"

source ${HELPER_DIR}/remote.sh

SSH_REMOTE="$(get_ssh_remote "$@")"
SSH_OPTS="${SSH_OPTS:-}"

[[ -d "${OVERLAYS_DIR}" ]] || { echo "Error: '${OVERLAYS_DIR}' directory not found."; exit 1; }

echo "🔄 Uploading overlays/ → ${SSH_REMOTE}:/ ..."
rsync -rptl --progress --partial -e "ssh -o ConnectTimeout=5 -o BatchMode=yes -o ConnectionAttempts=1 ${SSH_OPTS}" -r "${OVERLAYS_DIR}/home/pi/" "$SSH_REMOTE:./"
rsync -rlt --progress --partial -e "ssh -o ConnectTimeout=5 -o BatchMode=yes -o ConnectionAttempts=1 ${SSH_OPTS}" --rsync-path="sudo rsync" "${OVERLAYS_DIR}/" "$SSH_REMOTE:/" --exclude="home/"
echo "✅ Upload complete."

echo "🔨 Configuring scripts on Pi..."
ssh $SSH_OPTS "$SSH_REMOTE" "
    find /home/pi/scripts -type f -name '*.sh' -exec chmod +x {} \;
    
    ln -sf /home/pi/scripts/updater.sh /home/pi/bin/updater.sh
    ln -sf /home/pi/scripts/activate-update.sh /home/pi/bin/activate-update.sh
"
echo "✅ Sync finished. Scripts configured."
