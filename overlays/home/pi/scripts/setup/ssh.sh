#!/bin/bash

set -euo pipefail

SSHD_CONFIG="/etc/ssh/sshd_config"

# Ensure the file exists
if [ ! -f "$SSHD_CONFIG" ]; then
    echo "Error: $SSHD_CONFIG not found."
    exit 1
fi

# Uncomment or replace any existing PasswordAuthentication line
if grep -qE '^\s*#?\s*PasswordAuthentication\s+' "$SSHD_CONFIG"; then
    sudo sed -i 's/^\s*#\?\s*PasswordAuthentication\s\+.*/PasswordAuthentication yes/' "$SSHD_CONFIG"
else
    echo "PasswordAuthentication yes" | sudo tee -a "$SSHD_CONFIG" > /dev/null
fi

echo "✅ PasswordAuthentication enabled."
