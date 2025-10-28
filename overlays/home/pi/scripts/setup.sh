#!/usr/bin/bash
set -e

# A set of scripts called once after the overlays have been copied

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

#./setup/system.sh
#./setup/hostname.sh
#./setup/displayhatmini_workaround.sh # TODO remove once amaru-pi is self-contained
${SCRIPT_DIR}/setup/lean.sh
${SCRIPT_DIR}/setup/ssh.sh
${SCRIPT_DIR}/setup/py-displayhatmini.sh
${SCRIPT_DIR}/setup/amaru.sh
${SCRIPT_DIR}/setup/services.sh

echo "✅ Setup complete."

trap 'echo -e "\n❌ Reboot canceled by user."; exit 1' INT

read -rp "➡️ Reboot to apply changes? (yes/NO): " CONFIRM
if [[ "$CONFIRM" == "yes" ]]; then
    echo "Rebooting in 10 seconds... Press Ctrl+C to cancel."
    for i in {10..1}; do
        echo -ne "$i\r"
        sleep 1
    done
    echo "Rebooting now..."

    sudo reboot
    exit 1
fi