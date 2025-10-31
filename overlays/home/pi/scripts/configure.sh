#!/usr/bin/env bash

set -euo pipefail

${SCRIPT_DIR}/configure/set-envs.sh
${SCRIPT_DIR}/configure/set-wifi-conf.sh

echo "✅ Configuration complete."

ask_and_reboot