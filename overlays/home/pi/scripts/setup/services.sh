#!/bin/bash

set -euo pipefail

enable_service() {
    local service_name="$1"

    if [[ -z "$service_name" ]]; then
        echo "Usage: enable_and_start_service <service-name>"
        return 1
    fi

    # Use sudo only if not running as root
    local sudo_cmd=""
    if [[ $EUID -ne 0 ]]; then
        sudo_cmd="sudo"
    fi

    echo "Reloading systemd daemon..."
    $sudo_cmd systemctl daemon-reload

    echo "Checking if $service_name is enabled..."
    if ! systemctl is-enabled --quiet "$service_name"; then
        echo "Enabling $service_name..."
        $sudo_cmd systemctl enable "$service_name"
        echo "$service_name is now enabled."
    else
        echo "$service_name is already enabled."
    fi
}

enable_service amaru
enable_service amaru-pi
enable_service splash