#!/bin/bash

set -euo pipefail

die() { echo "FATAL: $1"; exit "${2:-1}"; }

cleanup() {
  echo "Running cleanup"
  if [ -n "${CONTROL_PATH:-}" ] && [ -S "${CONTROL_PATH}" ]; then
    echo "Closing SSH master connection"
    ssh -S "${CONTROL_PATH}" -O exit "${PI_SSH_TARGET}" >/dev/null 2>&1 || true
  fi
  if [ -n "${CONTROL_DIR:-}" ] && [ -d "${CONTROL_DIR}" ]; then
    rm -rf "${CONTROL_DIR}" || true
  fi
  echo "Cleanup complete"
}

trap 'rc=$?; echo "ERROR: trap triggered (rc=${rc})"; cleanup || true; exit $rc' ERR
trap 'cleanup' EXIT INT TERM

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)
PROJECT_ROOT=$(realpath "${SCRIPT_DIR}/../..")
ENV_FILE_PATH="${PROJECT_ROOT}/.env"

echo "Starting amaru-pi tty1 deployment"
[ -f "${ENV_FILE_PATH}" ] || die "Environment file not found at ${ENV_FILE_PATH}"
set -a
source "${ENV_FILE_PATH}"
set +a

: "${PI_HOST?FATAL: PI_HOST must be set in ${ENV_FILE_PATH}}"
: "${PI_USER?FATAL: PI_USER must be set in ${ENV_FILE_PATH}}"
: "${JUMP_PORT?FATAL: JUMP_PORT must be set in ${ENV_FILE_PATH}}"
if [ -z "${SSH_AUTH_SOCK:-}" ]; then
  die "SSH_AUTH_SOCK not set, enable SSH agent forwarding and add keys before running"
fi

JUMP_HOST=${JUMP_HOST:-localhost}
JUMP_USER=${JUMP_USER:-${USER}}
BUILD_TARGET="aarch64-unknown-linux-gnu"
APP_USER=${APP_USER:-pi}
APP_HOME="/home/${APP_USER}"

BINARY_NAME="amaru-pi"
LOCAL_BINARY_PATH="${PROJECT_ROOT}/app/target/${BUILD_TARGET}/release/${BINARY_NAME}"
STARTUP_SCRIPT_SOURCE="${PROJECT_ROOT}/overlays/usr/local/bin/amaru-pi-startup.sh"
HOTSPOT_SCRIPT_SOURCE="${PROJECT_ROOT}/overlays/usr/local/bin/amaru-hotspot.sh"
PROFILE_SCRIPT_SOURCE="${PROJECT_ROOT}/overlays/etc/profile.d/amaru-pi-tty1.sh"
HOTSPOT_SERVICE_SOURCE="${PROJECT_ROOT}/overlays/etc/systemd/system/amaru-hotspot.service"
HOTSPOT_TIMER_SOURCE="${PROJECT_ROOT}/overlays/etc/systemd/system/amaru-hotspot.timer"
GETTY_OVERRIDE_SOURCE="${PROJECT_ROOT}/overlays/etc/systemd/system/getty@tty1.service.d/override.conf"
SPLASH_SERVICE_SOURCE="${PROJECT_ROOT}/overlays/etc/systemd/system/splash.service"
RENDERED_STARTUP_SCRIPT=""

REMOTE_BINARY_PATH="${APP_HOME}/bin/${BINARY_NAME}"
REMOTE_STARTUP_SCRIPT="/usr/local/bin/amaru-pi-startup.sh"
REMOTE_HOTSPOT_SCRIPT="/usr/local/bin/amaru-hotspot.sh"
REMOTE_PROFILE_SCRIPT="/etc/profile.d/amaru-pi-tty1.sh"
REMOTE_HOTSPOT_SERVICE="/etc/systemd/system/amaru-hotspot.service"
REMOTE_HOTSPOT_TIMER="/etc/systemd/system/amaru-hotspot.timer"
REMOTE_GETTY_OVERRIDE="/etc/systemd/system/getty@tty1.service.d/override.conf"
REMOTE_SPLASH_SERVICE="/etc/systemd/system/splash.service"

SSH_OPTS="-o BatchMode=yes -o ConnectTimeout=10 -o ServerAliveInterval=30 -o ServerAliveCountMax=3"
SSH_JUMP_OPTS="-o ProxyJump=${JUMP_USER}@${JUMP_HOST}:${JUMP_PORT}"
PI_SSH_TARGET="${PI_USER}@${PI_HOST}"

CONTROL_DIR="$(mktemp -d "${TMPDIR:-/tmp}"/remote-deploy.XXXXXXXX)" || die "mktemp for CONTROL_DIR failed"
CONTROL_PATH="${CONTROL_DIR}/ssh_control"

ssh_on_pi() {
  echo "Executing on pi"
  ssh ${SSH_OPTS} ${SSH_JUMP_OPTS} -S "${CONTROL_PATH}" "${PI_SSH_TARGET}" 'bash -s'
}

upload_file() {
  local local_path="$1"
  local remote_path="$2"

  echo "Uploading ${local_path}"
  scp ${SSH_OPTS} ${SSH_JUMP_OPTS} -o "ControlPath=${CONTROL_PATH}" "${local_path}" "${PI_SSH_TARGET}:${remote_path}"
}

install_owned_file() {
  local local_path="$1"
  local remote_path="$2"
  local mode="$3"
  local owner="$4"
  local group="$5"
  local remote_tmp="/tmp/$(basename "${remote_path}").$$"

  upload_file "${local_path}" "${remote_tmp}"
  ssh_on_pi <<EOF
set -euo pipefail
sudo install -D -m '${mode}' -o '${owner}' -g '${group}' '${remote_tmp}' '${remote_path}'
rm -f '${remote_tmp}'
EOF
}

render_startup_script() {
  RENDERED_STARTUP_SCRIPT="${CONTROL_DIR}/amaru-pi-startup.sh"
  sed -e "s,/home/pi,${APP_HOME},g" "${STARTUP_SCRIPT_SOURCE}" > "${RENDERED_STARTUP_SCRIPT}"
}

echo "Establishing master SSH connection to ${PI_HOST}"
ssh ${SSH_OPTS} ${SSH_JUMP_OPTS} -M -N -f -S "${CONTROL_PATH}" "${PI_SSH_TARGET}" || die "Failed to establish SSH master connection"
echo "Master connection established"

echo "Validating local files"
for f in \
  "${ENV_FILE_PATH}" \
  "${PROJECT_ROOT}/app/Makefile" \
  "${STARTUP_SCRIPT_SOURCE}" \
  "${HOTSPOT_SCRIPT_SOURCE}" \
  "${PROFILE_SCRIPT_SOURCE}" \
  "${HOTSPOT_SERVICE_SOURCE}" \
  "${HOTSPOT_TIMER_SOURCE}" \
  "${GETTY_OVERRIDE_SOURCE}" \
  "${SPLASH_SERVICE_SOURCE}"; do
  if [ ! -f "$f" ]; then
    die "Missing local file: $f"
  fi
done
echo "Local file checks OK"

render_startup_script

echo "Building binary '${BINARY_NAME}' for ${BUILD_TARGET}"
( cd "${PROJECT_ROOT}/app" && make build )
[ -f "${LOCAL_BINARY_PATH}" ] || die "Build did not produce expected binary at ${LOCAL_BINARY_PATH}"
echo "Build complete"

echo "Preparing remote directories"
ssh_on_pi <<EOF
set -euo pipefail
sudo install -d -m 0755 -o '${APP_USER}' -g '${APP_USER}' '${APP_HOME}/bin'
sudo install -d -m 0755 '/etc/systemd/system/getty@tty1.service.d'
EOF

echo "Stopping existing amaru-pi runtime"
ssh_on_pi <<EOF
set -euo pipefail
sudo systemctl stop amaru-pi.service || true
sudo systemctl disable amaru-pi.service || true
sudo systemctl stop getty@tty1.service || true
sudo pkill -f '${REMOTE_BINARY_PATH}' || true
EOF

echo "Installing binary and tty1 runtime assets"
install_owned_file "${LOCAL_BINARY_PATH}" "${REMOTE_BINARY_PATH}" 0755 "${APP_USER}" "${APP_USER}"
install_owned_file "${RENDERED_STARTUP_SCRIPT}" "${REMOTE_STARTUP_SCRIPT}" 0755 root root
install_owned_file "${HOTSPOT_SCRIPT_SOURCE}" "${REMOTE_HOTSPOT_SCRIPT}" 0755 root root
install_owned_file "${PROFILE_SCRIPT_SOURCE}" "${REMOTE_PROFILE_SCRIPT}" 0644 root root
install_owned_file "${HOTSPOT_SERVICE_SOURCE}" "${REMOTE_HOTSPOT_SERVICE}" 0644 root root
install_owned_file "${HOTSPOT_TIMER_SOURCE}" "${REMOTE_HOTSPOT_TIMER}" 0644 root root
install_owned_file "${GETTY_OVERRIDE_SOURCE}" "${REMOTE_GETTY_OVERRIDE}" 0644 root root
install_owned_file "${SPLASH_SERVICE_SOURCE}" "${REMOTE_SPLASH_SERVICE}" 0644 root root

echo "Reloading systemd and starting tty1 runtime"
ssh_on_pi <<EOF
set -euo pipefail
sudo systemctl disable --now updater.timer updater.service activate-update.path activate-update.service || true
sudo rm -f /etc/systemd/system/updater.service /etc/systemd/system/updater.timer
sudo rm -f /etc/systemd/system/activate-update.service /etc/systemd/system/activate-update.path
sudo rm -f '${APP_HOME}/bin/updater.sh' '${APP_HOME}/bin/activate-update.sh'
sudo rm -f '${APP_HOME}/scripts/updater.sh' '${APP_HOME}/scripts/activate-update.sh' '${APP_HOME}/scripts/start-amaru.sh'
sudo rm -f '${APP_HOME}/.amaru_update_state.json' '${APP_HOME}/.update_requested'
if sudo test -f /etc/systemd/system/amaru.service && sudo grep -q '/home/pi/scripts/start-amaru.sh' /etc/systemd/system/amaru.service; then
  sudo sed -i 's#ExecStart=/home/pi/scripts/start-amaru.sh#ExecStart=/home/pi/bin/amaru daemon#' /etc/systemd/system/amaru.service
fi
sudo systemctl daemon-reload
sudo ${REMOTE_HOTSPOT_SCRIPT} ensure
sudo systemctl enable amaru-hotspot.timer
sudo systemctl restart amaru-hotspot.timer
sudo systemctl enable splash.service
sudo systemctl enable getty@tty1.service
sudo systemctl restart getty@tty1.service
EOF

echo "Verifying '${BINARY_NAME}' process is running (timeout: 15s)"
VERIFY_SUCCESS=false
for i in {1..15}; do
  if ssh_on_pi <<< "pgrep -f '${REMOTE_BINARY_PATH}'" >/dev/null 2>&1; then
    VERIFY_SUCCESS=true
    echo "'${BINARY_NAME}' process is running on ${PI_HOST} after ${i} seconds"
    break
  fi
  echo "Waiting, attempt ${i}/15"
  sleep 1
done

if [ "${VERIFY_SUCCESS}" = false ]; then
  die "'${BINARY_NAME}' process did not start after deploy"
fi

echo "Deployment complete"