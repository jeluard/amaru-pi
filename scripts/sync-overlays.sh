#!/usr/bin/env bash
set -euo pipefail

SSH_REMOTE="${SSH_REMOTE:-$1}"
SSH_OPTS="${SSH_OPTS:-}"
OVERLAYS_DIR="overlays"
SETUP_SCRIPT="/home/pi/scripts/setup.sh"
RUN_SETUP="${RUN_SETUP:-true}"

if [[ -z "$SSH_REMOTE" ]]; then
  echo "Usage: $0 user@host"
  exit 2
fi

[[ -d "${OVERLAYS_DIR}" ]] || { echo "Error: '${OVERLAYS_DIR}' directory not found."; exit 1; }

echo "🔄 Uploading overlays/ → ${SSH_REMOTE}:/ ..."
rsync -rptl --progress --partial -e "ssh $SSH_OPTS" -r "${OVERLAYS_DIR}/home" "$SSH_REMOTE:/"
rsync -rptl --progress --partial -e "ssh $SSH_OPTS" --rsync-path="sudo rsync" "${OVERLAYS_DIR}/" "$SSH_REMOTE:/" --exclude="home/"
echo "✅ Upload complete."

if [[ -n "$RUN_SETUP" ]]; then
  echo "🔨 Running remote setup script: $SETUP_SCRIPT"
  ssh $SSH_OPTS "$SSH_REMOTE" "SETUP_SCRIPT='$SETUP_SCRIPT'; if [[ -f \$SETUP_SCRIPT ]]; then sudo bash \"\$SETUP_SCRIPT\"; else echo '⚠️  Script '\$SETUP_SCRIPT' not found'; fi"
else
  echo "⚠️ Skipping $SETUP_SCRIPT (RUN_SETUP not set)"
fi

echo "✅ Sync finished. Reboot to apply changes."
