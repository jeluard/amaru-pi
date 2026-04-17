#!/usr/bin/env bash

set -euo pipefail

HELPER_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(realpath "${HELPER_DIR}/../..")"
OVERLAYS_DIR="${ROOT_DIR}/overlays"

source ${HELPER_DIR}/remote.sh

SSH_REMOTE="$(get_ssh_remote "$@")"
SSH_OPTS="${SSH_OPTS:-}"

[[ -d "${OVERLAYS_DIR}" ]] || { echo "Error: '${OVERLAYS_DIR}' directory not found."; exit 1; }

mkdir -p ~/.ssh/cm
chmod 700 ~/.ssh/cm

export RSYNC_SSH="ssh -o ConnectTimeout=5 -o ConnectionAttempts=1 \
  -o ControlMaster=auto \
  -o ControlPath=~/.ssh/cm/%r@%h:%p \
  -o ControlPersist=10m"

echo "🔄 Uploading overlays/ → ${SSH_REMOTE}:/ ..."
rsync -rptl --progress --partial -e "${RSYNC_SSH} -o ConnectTimeout=5 -o ConnectionAttempts=1 ${SSH_OPTS}" -r "${OVERLAYS_DIR}/home/pi/" "$SSH_REMOTE:./"
rsync -rlt --progress --partial -e "${RSYNC_SSH} -o ConnectTimeout=5 -o ConnectionAttempts=1 ${SSH_OPTS}" --rsync-path="sudo rsync" "${OVERLAYS_DIR}/" "$SSH_REMOTE:/" --exclude="home/"
echo "✅ Upload complete."

echo "🔨 Configuring scripts on Pi..."
ssh ${SSH_OPTS} "${SSH_REMOTE}" "
  set -euo pipefail
    find /home/pi/scripts -type f -name '*.sh' -exec chmod +x {} \;

  sudo systemctl disable --now updater.timer updater.service activate-update.path activate-update.service >/dev/null 2>&1 || true
  rm -f /home/pi/bin/updater.sh /home/pi/bin/activate-update.sh
  rm -f /home/pi/scripts/updater.sh /home/pi/scripts/activate-update.sh /home/pi/scripts/start-amaru.sh
  rm -f /home/pi/.amaru_update_state.json /home/pi/.update_requested
  sudo rm -f /etc/systemd/system/updater.service /etc/systemd/system/updater.timer
  sudo rm -f /etc/systemd/system/activate-update.service /etc/systemd/system/activate-update.path
  if sudo test -f /etc/systemd/system/amaru.service && sudo grep -q '/home/pi/scripts/start-amaru.sh' /etc/systemd/system/amaru.service; then
    sudo sed -i 's#ExecStart=/home/pi/scripts/start-amaru.sh#ExecStart=/home/pi/bin/amaru daemon#' /etc/systemd/system/amaru.service
  fi
  sudo systemctl daemon-reload
"
echo "✅ Sync finished. Scripts configured."
