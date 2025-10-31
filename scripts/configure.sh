#!/usr/bin/env bash

# A simple script to remotely execute configure-pi.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

source ${SCRIPT_DIR}/helper/remote.sh

SSH_REMOTE="${SSH_REMOTE:-$1}"

if [[ -z "$SSH_REMOTE" ]]; then
  echo "Usage: $0 user@host"
  exit 2
fi

if [ -z "${!AMARU_WORDS}" ]; then
    echo "Error: $AMARU_WORDS is not set."
    exit 1
fi

if [ -z "${AMARU_WIFI_SSID:-}" ]; then
    echo "Error: $AMARU_WIFI_SSID is not set."
    exit 1
fi

if [ -z "${AMARU_WIFI_PASSWORD:-}" ]; then
    echo "Error: $AMARU_WIFI_PASSWORD is not set."
    exit 1
fi

run_remote_script ${SSH_REMOTE} ${SSH_OPTS} /home/pi/scripts/configure.sh AMARU_WORDS AMARU_WIFI_SSID AMARU_WIFI_PASSWORD
rc=$?

if [[ $rc -ne 0 ]]; then
    echo "❌ Remote configuration failed with code $rc"
    exit $rc
fi

echo "✅ Remote configuration succeeded"
