#!/usr/bin/env bash

set -euo pipefail

if [ -z "${AMARU_WIFI_SSID:-}" ]; then
    echo "Error: $AMARU_WIFI_SSID is not set."
    exit 1
fi

if [ -z "${AMARU_WIFI_PASSWORD:-}" ]; then
    echo "Error: $AMARU_WIFI_PASSWORD is not set."
    exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

source ${SCRIPT_DIR}/helper.sh

amaru_pi_conf wifi set-connection $AMARU_WIFI_SSID $AMARU_WIFI_PASSWORD