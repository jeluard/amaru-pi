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

echo "Starting doctor TUI deployment"
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
BUILD_TARGET=${BUILD_TARGET:-aarch64-unknown-linux-musl}

BINARY_NAME="doctor"
PI_DEST_PATH="/home/${PI_USER}/amaru-pi"
STARTUP_SCRIPT_NAME="doctor-startup.sh"
STARTUP_SCRIPT_PATH="${SCRIPT_DIR}/${STARTUP_SCRIPT_NAME}"
BASH_PROFILE_LINE_TEMPLATE_PATH="${SCRIPT_DIR}/bash_profile.line.template"
GETTY_OVERRIDE_TEMPLATE_PATH="${SCRIPT_DIR}/getty-override.conf.template"
LOCAL_BINARY_PATH="${PROJECT_ROOT}/app/target/${BUILD_TARGET}/release/${BINARY_NAME}"

SSH_OPTS="-o BatchMode=yes -o ConnectTimeout=10 -o ServerAliveInterval=30 -o ServerAliveCountMax=3"
SSH_JUMP_OPTS="-o ProxyJump=${JUMP_USER}@${JUMP_HOST}:${JUMP_PORT}"
PI_SSH_TARGET="${PI_USER}@${PI_HOST}"

CONTROL_DIR="$(mktemp -d "${TMPDIR:-/tmp}"/remote-deploy.XXXXXXXX)" || die "mktemp for CONTROL_DIR failed"
CONTROL_PATH="${CONTROL_DIR}/ssh_control"

ssh_on_pi() {
    echo "Executing on pi"
    ssh ${SSH_OPTS} ${SSH_JUMP_OPTS} -S "${CONTROL_PATH}" "${PI_SSH_TARGET}" 'bash -s'
}

ssh_on_pi_with_tty() {
    echo "Executing pi tty"
    ssh -t ${SSH_OPTS} ${SSH_JUMP_OPTS} -S "${CONTROL_PATH}" "${PI_SSH_TARGET}" 'bash -s'
}

echo "Establishing master SSH connection to ${PI_HOST}"
ssh ${SSH_OPTS} ${SSH_JUMP_OPTS} -M -N -f -S "${CONTROL_PATH}" "${PI_SSH_TARGET}" || die "Failed to establish SSH master connection"
echo "Master connection established"

echo "Validating local files"
for f in "${ENV_FILE_PATH}" "${STARTUP_SCRIPT_PATH}" "${BASH_PROFILE_LINE_TEMPLATE_PATH}" "${GETTY_OVERRIDE_TEMPLATE_PATH}" "${PROJECT_ROOT}/app/Cargo.toml"; do
  if [ ! -f "$f" ]; then
    die "Missing local file: $f"
  fi
done
echo "Local file checks OK"

echo "Validating remote ~/.bash_profile"
REQUIRED_LINE=$(sed -e "s,{{PI_DEST_PATH}},${PI_DEST_PATH},g" -e "s,{{STARTUP_SCRIPT_NAME}},${STARTUP_SCRIPT_NAME},g" "${BASH_PROFILE_LINE_TEMPLATE_PATH}")
ssh_on_pi <<EOF
set -ex
REQUIRED_LINE_REMOTE='${REQUIRED_LINE}'
if ! grep -qF -- "\${REQUIRED_LINE_REMOTE}" ~/.bash_profile 2>/dev/null; then
    echo '~/.bash_profile is missing the required line, adding it'
    printf '%s\\n' "\${REQUIRED_LINE_REMOTE}" >> ~/.bash_profile
fi
EOF
echo "~/.bash_profile check complete"

echo "Validating remote getty override file"
GETTY_OVERRIDE_PATH="/etc/systemd/system/getty@tty1.service.d/override.conf"
EXPECTED_GETTY_CONTENT=$(sed -e "s/{{PI_USER}}/${PI_USER}/g" "${GETTY_OVERRIDE_TEMPLATE_PATH}")

# This entire block is sent to the remote shell, avoiding local syntax errors.
ssh_on_pi_with_tty <<EOF
set -ex
GETTY_OVERRIDE_PATH_REMOTE='${GETTY_OVERRIDE_PATH}'
EXPECTED_GETTY_CONTENT_REMOTE='${EXPECTED_GETTY_CONTENT}'
# Note: '\$' ensures mktemp and dirname run on the remote host.
TMPFILE=\$(mktemp)
# The trap ensures the temp file is removed even if the script fails.
trap 'rm -f "\${TMPFILE}"' EXIT

printf '%s' "\${EXPECTED_GETTY_CONTENT_REMOTE}" > "\${TMPFILE}"

# If cmp fails (files differ) or the destination doesn't exist, update it.
if ! sudo cmp -s "\${TMPFILE}" "\${GETTY_OVERRIDE_PATH_REMOTE}" 2>/dev/null; then
    echo "   -> Getty override config is incorrect or missing. Recreating it..."
    sudo mkdir -p "\$(dirname "\${GETTY_OVERRIDE_PATH_REMOTE}")"
    sudo mv "\${TMPFILE}" "\${GETTY_OVERRIDE_PATH_REMOTE}"
    sudo chown root:root "\${GETTY_OVERRIDE_PATH_REMOTE}"
    echo "   -> Reloading systemd and restarting getty service..."
    sudo systemctl daemon-reload
    sudo systemctl restart getty@tty1.service
else
    echo "   -> Getty override config is already correct."
fi
EOF
echo "getty override check complete."

echo "Building binary '${BINARY_NAME}' for ${BUILD_TARGET}"
( cd "${PROJECT_ROOT}/app" && RUSTFLAGS=-Awarnings cross build --quiet --release --bin "${BINARY_NAME}" --target "${BUILD_TARGET}" )
[ -f "${LOCAL_BINARY_PATH}" ] || die "Build did not produce expected binary at ${LOCAL_BINARY_PATH}"
echo "Build complete"

echo "Deploying files to ${PI_HOST}"
echo "Stopping getty service and killing any lingering TUI processes"
ssh_on_pi_with_tty <<< "set -ex; sudo systemctl stop getty@tty1.service || true; sudo pkill -f \"${PI_DEST_PATH}/${BINARY_NAME}\" || true"
echo "Shutdown command executed"

echo "Creating destination directory on Pi"
ssh_on_pi <<< "set -ex; mkdir -p '${PI_DEST_PATH}'"
echo "Destination directory ensured."

echo "Deploying new files"
upload_file() {
    local local_path="$1"
    local remote_path="$2"
    echo "Uploading ${local_path}"
    scp ${SSH_OPTS} ${SSH_JUMP_OPTS} -o "ControlPath=${CONTROL_PATH}" "${local_path}" "${PI_SSH_TARGET}:${remote_path}"
}
upload_file "${LOCAL_BINARY_PATH}" "${PI_DEST_PATH}/${BINARY_NAME}"
upload_file "${STARTUP_SCRIPT_PATH}" "${PI_DEST_PATH}/${STARTUP_SCRIPT_NAME}"
echo "Files transferred"

echo "Setting file permissions on Pi"
ssh_on_pi <<< "set -ex; chmod +x '${PI_DEST_PATH}/${BINARY_NAME}' '${PI_DEST_PATH}/${STARTUP_SCRIPT_NAME}'"
echo "Permissions set"

echo "Restarting service and verifying TUI process"
echo "Starting getty@tty1.service"
ssh_on_pi_with_tty <<< "set -ex; sudo systemctl start getty@tty1.service"
echo "Start command sent successfully"

echo "Verifying '${BINARY_NAME}' process is running (timeout: 15s)"
VERIFY_SUCCESS=false
for i in {1..15}; do
  if ssh_on_pi <<< "pgrep -f '${PI_DEST_PATH}/${BINARY_NAME}'" >/dev/null 2>&1; then
    VERIFY_SUCCESS=true
    echo "'${BINARY_NAME}' process is running on ${PI_HOST} after ${i} seconds"
    break
  fi
  echo "Waiting, attempt ${i}/15"
  sleep 1
done

if [ "${VERIFY_SUCCESS}" = false ]; then
  echo "'${BINARY_NAME}' process did not start after deploy"
  die "'${BINARY_NAME}' process did not start after deploy"
fi

echo "Deployment complete"