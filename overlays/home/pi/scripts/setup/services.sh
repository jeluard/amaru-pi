#!/bin/bash

set -euo pipefail

run_cmd() {
    if [[ $EUID -ne 0 ]]; then
        sudo "$@"
    else
        "$@"
    fi
}

reload_systemd() {
    echo "🔁 Reloading systemd daemon..."
    run_cmd systemctl daemon-reload
}

enable_service() {
    local service_name="$1"

    if [[ -z "$service_name" ]]; then
        echo "Usage: enable_and_start_service <service-name>"
        return 1
    fi

    if ! systemctl is-enabled --quiet "$service_name"; then
        echo "➡️ Enabling $service_name..."
        run_cmd systemctl enable "$service_name"
    fi
}

disable_service() {
    local service_name="$1"

    if [[ -z "$service_name" ]]; then
        echo "Usage: disable_service <service-name>"
        return 1
    fi

    if systemctl is-enabled --quiet "$service_name"; then
        echo "⛔ Disabling $service_name..."
        run_cmd systemctl disable "$service_name"
    fi

    if systemctl is-active --quiet "$service_name"; then
        echo "🛑 Stopping $service_name..."
        run_cmd systemctl stop "$service_name"
    fi
}

reload_systemd
enable_service first-boot
enable_service amaru
enable_service amaru-hotspot.timer
enable_service splash
enable_service getty@tty1.service
enable_service updater.timer
enable_service activate-update.path
disable_service amaru-pi
disable_service hotspot-nat
