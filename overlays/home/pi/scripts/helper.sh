#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_DIR=${SCRIPT_DIR}/../bin

ask_and_reboot() {
    trap 'echo -e "\n❌ Reboot canceled by user."; exit 1' INT

    read -rp "➡️ Reboot to apply changes? (yes/NO): " CONFIRM
    if [[ "$CONFIRM" == "yes" ]]; then
        echo "➡️ Rebooting in 10 seconds... Press Ctrl+C to cancel."
        for i in {10..1}; do
            echo -ne "$i\r"
            sleep 1
        done
        echo "🔁 Rebooting now..."

        sudo reboot
    else
        echo "⚠️ No reboot. Changes won't be applied until next reboot"
    fi
}

amaru_pi_conf() {
    # Default arguments
    local default_args="conf"

    # Call the binary with default args plus any extra ones passed to the function
    sudo ${BIN_DIR}/amaru-pi $default_args "$@"
}