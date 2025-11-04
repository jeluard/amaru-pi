#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
HELPER_DIR="${SCRIPT_DIR}/helper"

source ${SCRIPT_DIR}/helper/remote.sh

SSH_REMOTE="$(get_ssh_remote "$@")"
SSH_OPTS="${SSH_OPTS:-}"
BUILD_ASSETS="${BUILD_ASSETS:-false}"
RUN_SETUP="${RUN_SETUP:-true}"

if [[ -n "$BUILD_ASSETS" ]]; then
  ${HELPER_DIR}/build-assets.sh
else
  echo "⚠️ Skipping building assets (BUILD_ASSETS not set)"
fi

${HELPER_DIR}/sync-overlays.sh

run_remote_script {$SSH_REMOTE} ${SSH_OPTS} /home/pi/scripts/setup.sh AMARU_WORDS AMARU_WIFI_SSID AMARU_WIFI_PASSWORD
rc=$?

if [[ $rc -ne 0 ]]; then
    echo "❌ Remote setup failed with code $rc"
    exit $rc
fi

echo "✅ Remote setup succeeded"
