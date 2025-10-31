#!/usr/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

source ${SCRIPT_DIR}/helper.sh

${SCRIPT_DIR}/setup/lean.sh
${SCRIPT_DIR}/setup/ssh.sh
${SCRIPT_DIR}/setup/py-displayhatmini.sh
${SCRIPT_DIR}/setup/amaru.sh
${SCRIPT_DIR}/setup/services.sh

echo "✅ Setup complete."

ask_and_reboot