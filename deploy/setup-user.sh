#!/bin/bash

set -euo pipefail

DHM_REPO_URL="https://github.com/pimoroni/displayhatmini-python.git"
DHM_REPO_DIR="displayhatmini-python"

check_not_root() {
    echo "Checking for user privileges"
    if [[ "${EUID}" -eq 0 ]]; then
        echo "Error: This script must not be run as root, please run it as your regular user" >&2
        exit 1
    fi
    echo "Running as user '$(whoami)'"
}

setup_dhm() {
    echo "Setting up Python project"

    if [ -d "${DHM_REPO_DIR}" ]; then
        echo "Repository directory '${DHM_REPO_DIR}' already exists, skipping clone"
    else
        echo "Cloning repository into '${DHM_REPO_DIR}'"
        git clone "${REPO_URL}"
    fi

    cd "${DHM_REPO_DIR}"

    if [ -d ".venv" ]; then
        echo "Venv already exists, skipping creation."
    else
        echo "Creating Python virtual environment in .venv"
        python3 -m venv .venv
    fi

    echo "Activating virtual environment and installing Python packages"
    source ".venv/bin/activate"
    
    echo "Upgrading pip"
    pip install --upgrade pip
    
    echo "Installing pygame, displayhatmini, and rpi-lgpio"
    pip install pygame displayhatmini rpi-lgpio
    
    echo "Verifying Python packages are installed"
    pip show pygame displayhatmini rpi-lgpio > /dev/null
    echo "All Python packages installed successfully"

    deactivate
    echo "Virtual environment setup complete"
    cd ..
}

main() {
    check_not_root
    setup_dhm

    echo "User environment setup is complete"
    echo "Navigate to the '${DHM_REPO_DIR}' directory to run the examples"
    echo "Activate the environment first with: source ${DHM_REPO_DIR}/${VENV_DIR}/bin/activate"
}

main
