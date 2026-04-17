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

remove_path() {
    local path="$1"

    if [[ -e "$path" || -L "$path" ]]; then
        echo "🧹 Removing $path..."
        run_cmd rm -f "$path"
    fi
}

restore_amaru_service_execstart() {
    local service_path="/etc/systemd/system/amaru.service"

    if [[ -f "$service_path" ]] && grep -q '/home/pi/scripts/start-amaru.sh' "$service_path"; then
        echo "↩️ Restoring amaru.service ExecStart..."
        run_cmd sed -i 's#ExecStart=/home/pi/scripts/start-amaru.sh#ExecStart=/home/pi/bin/amaru daemon#' "$service_path"
    fi
}

restore_amaru_service_execstart
reload_systemd
enable_service first-boot
enable_service amaru
enable_service amaru-hotspot.timer
enable_service splash
enable_service getty@tty1.service
disable_service updater.timer
disable_service updater.service
disable_service activate-update.path
disable_service activate-update.service
disable_service amaru-pi
disable_service hotspot-nat
remove_path /etc/systemd/system/updater.service
remove_path /etc/systemd/system/updater.timer
remove_path /etc/systemd/system/activate-update.service
remove_path /etc/systemd/system/activate-update.path
remove_path /home/pi/bin/updater.sh
remove_path /home/pi/bin/activate-update.sh
remove_path /home/pi/scripts/updater.sh
remove_path /home/pi/scripts/activate-update.sh
remove_path /home/pi/scripts/start-amaru.sh
remove_path /home/pi/.amaru_update_state.json
remove_path /home/pi/.update_requested
reload_systemd
