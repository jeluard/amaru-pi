#!/usr/bin/bash
set -e

# A set of scripts called once after the overlays have been copied

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

#./setup/system.sh
#./setup/hostname.sh
#./setup/displayhatmini_workaround.sh # TODO remove once amaru-pi is self-contained
${SCRIPT_DIR}/setup/lean.sh
${SCRIPT_DIR}/setup/py-displayhatmini.sh
${SCRIPT_DIR}/setup/amaru.sh
${SCRIPT_DIR}/setup/services.sh

echo "✅ Setup complete."