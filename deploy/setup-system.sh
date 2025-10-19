#!/bin/bash

set -euo pipefail

CONFIG_FILE="/boot/firmware/config.txt"

line_exists_in_file() {
    grep -Fxq "$1" "$2"
}

check_for_root() {
    echo "Checking for root privileges"
    if [[ "${EUID}" -ne 0 ]]; then
        echo "Error: This script must be run as root, please use sudo" >&2
        exit 1
    fi
    echo "Root check passed"
}

update_system() {
    echo "Updating package lists and upgrading system"
    apt-get update
    DEBIAN_FRONTEND=noninteractive apt-get upgrade -y
    echo "System update complete"
}

install_dependencies() {
    echo "Installing system dependencies"
    apt-get remove -y python3-rpi.gpio || true
    
    DEBIAN_FRONTEND=noninteractive apt-get install -y \
        mosh \
        vim \
        python3-dev \
        swig \
        libgpiod-dev \
        liblgpio-dev \
        git \
        python3-venv
    echo "Dependency installation complete"
}

configure_boot() {
    echo "Configuring boot options in ${CONFIG_FILE}"
    
    local dtoverlay="dtoverlay=displayhatmini"
    local dtparam="dtparam=spi=on"

    if ! line_exists_in_file "${dtoverlay}" "${CONFIG_FILE}"; then
        echo "Adding '${dtoverlay}' to ${CONFIG_FILE}"
        echo "${dtoverlay}" | tee -a "${CONFIG_FILE}"
    else
        echo "'${dtoverlay}' already exists in ${CONFIG_FILE}"
    fi

    if ! line_exists_in_file "${dtparam}" "${CONFIG_FILE}"; then
        echo "Adding '${dtparam}' to ${CONFIG_FILE}"
        echo "${dtparam}" | tee -a "${CONFIG_FILE}"
    else
        echo "'${dtparam}' already exists in ${CONFIG_FILE}"
    fi

}

main() {
    check_for_root

    local target_user="${1:-${SUDO_USER:-}}"
    if [[ -z "${target_user}" ]]; then
        echo "Error: Could not determine target user, please specify a username" >&2
        echo "Usage: sudo $0 <username>" >&2
        exit 1
    fi

    update_system
    install_dependencies
    configure_boot

    echo "System setup complete"
    echo "A REBOOT IS REQUIRED for all changes (boot config and user permissions) to take effect"
    echo "After rebooting, run the user setup script: ./setup_user.sh"
    
    reboot
}

main "$@"