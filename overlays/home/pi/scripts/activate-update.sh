#!/bin/bash
set -euo pipefail

STATE_FILE="/home/pi/.amaru_update_state.json"
BIN_DIR="/home/pi/bin"
TRIGGER_FILE="/home/pi/update-requested"
LOCK_FILE="/tmp/amaru_update.lock"

exec 200>"$LOCK_FILE"
flock -n 200 || { echo "ERROR: Another update is in progress."; exit 1; }

log() { logger -t amaru-update "$1"; echo "$1"; }

abort() {
    log "ERROR: $1"
    rm -f "$TRIGGER_FILE"
    exit 1
}

validate_state_file() {
    if [ ! -f "$STATE_FILE" ]; then
        abort "State file ${STATE_FILE} not found."
    fi
    if ! jq empty "$STATE_FILE" 2>/dev/null; then
        abort "Invalid JSON in ${STATE_FILE}."
    fi
}

stop_service() {
    log "INFO: Stopping core service..."
    if ! systemctl stop amaru-pi.service; then
        abort "Failed to stop amaru-pi.service."
    fi
}

backup_and_swap_binary() {
    local app_name="$1"
    local staged_path="$2"

    local backup_path="${BIN_DIR}/${app_name}.bak.$(date +%s)"
    if [ -f "${BIN_DIR}/${app_name}" ]; then
        mv "${BIN_DIR}/${app_name}" "$backup_path"
        log "INFO: Backed up ${app_name} to ${backup_path}"
    fi

    log "INFO: Swapping binary for ${app_name}..."
    mv "$staged_path" "${BIN_DIR}/${app_name}"
    chmod +x "${BIN_DIR}/${app_name}"
}

apply_updates() {
    local state_json="$1"

    for app_name in $(echo "$state_json" | jq -r '.applications | keys[]'); do
        local update_available
        update_available=$(echo "$state_json" | jq -r ".applications[\"${app_name}\"].update_available")
        local staged_path
        staged_path=$(echo "$state_json" | jq -r ".applications[\"${app_name}\"].staged_path")

        if [ "$update_available" == "true" ] && [ -n "$staged_path" ]; then
            if [ -f "$staged_path" ]; then
                backup_and_swap_binary "$app_name" "$staged_path"
            else
                log "WARN: Staged file ${staged_path} for ${app_name} not found. Skipping."
            fi
        fi
    done
}

reset_update_state() {
    local state_json="$1"
    log "INFO: Resetting update state in JSON..."
    local temp_state_file
    temp_state_file=$(mktemp)
    echo "$state_json" | jq '
        .notify_after = 0 |
        .applications |= with_entries(
            .value.update_available = false |
            .value.staged_path = ""
        )
    ' > "$temp_state_file"
    mv "$temp_state_file" "$STATE_FILE"
    chown pi:pi "$STATE_FILE"
}

start_service() {
    log "INFO: Starting core service..."
    if ! systemctl start amaru-pi.service; then
        abort "Failed to start amaru-pi.service. Manual intervention required."
    fi
}

main() {
    log "INFO: Update activation triggered."
    validate_state_file
    local state_json
    state_json=$(cat "$STATE_FILE")
    stop_service
    apply_updates "$state_json"
    reset_update_state "$state_json"
    rm -f "$TRIGGER_FILE"
    start_service
    log "INFO: Update activation complete."
}

main "$@"
