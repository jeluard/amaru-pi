#!/bin/bash
set -euo pipefail

STATE_FILE="/home/pi/.amaru_update_state.json"
BIN_DIR="/home/pi/bin"
TRIGGER_FILE="/home/pi/.update_requested"
LOCK_FILE="/tmp/amaru_update.lock"

declare -a MANAGED_SERVICES=("amaru-pi.service")

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

stop_services() {
    log "INFO: Stopping all managed services..."
    for service in "${MANAGED_SERVICES[@]}"; do
        if ! systemctl stop "$service"; then
            log "WARN: Failed to stop $service. Continuing..."
        fi
    done
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

apply_updates_and_promote_versions() {
    local state_json="$1"
    local new_state_json="$state_json" # Start with the original state

    for app_name in $(echo "$state_json" | jq -r '.applications | keys[]'); do
        local pending_version
        pending_version=$(echo "$state_json" | jq -r ".applications[\"${app_name}\"].pending_version")
        local staged_path
        staged_path=$(echo "$state_json" | jq -r ".applications[\"${app_name}\"].staged_path")

        # Check if an update is pending
        if [ -n "$pending_version" ] && [ -n "$staged_path" ]; then
            if [ -f "$staged_path" ]; then
                log "INFO: Applying update for ${app_name} version ${pending_version}..."
                backup_and_swap_binary "$app_name" "$staged_path"
                
                # Update the JSON state for this app (promotion)
                new_state_json=$(echo "$new_state_json" | jq \
                    ".applications[\"${app_name}\"].current_version = \"${pending_version}\" |
                     .applications[\"${app_name}\"].pending_version = \"\" |
                     .applications[\"${app_name}\"].staged_path = \"\"")
            else
                log "WARN: Staged file ${staged_path} for ${app_name} not found. Skipping."
                # Clear the pending state so we don't try again
                new_state_json=$(echo "$new_state_json" | jq \
                    ".applications[\"${app_name}\"].pending_version = \"\" |
                     .applications[\"${app_name}\"].staged_path = \"\"")
            fi
        fi
    done
    
    # Reset notify_after and write the new state all at once
    log "INFO: Resetting update state in JSON..."
    local temp_state_file
    temp_state_file=$(mktemp)
    
    echo "$new_state_json" | jq '.notify_after = 0' > "$temp_state_file"
    
    mv "$temp_state_file" "$STATE_FILE"
    chown pi:pi "$STATE_FILE"
}

start_services() {
    log "INFO: Starting all managed services..."
    for service in "${MANAGED_SERVICES[@]}"; do
        if ! systemctl start "$service"; then
            log "ERROR: Failed to start $service. Manual intervention may be required."
        fi
    done
}

main() {
    log "INFO: Update activation triggered."
    validate_state_file
    local state_json
    state_json=$(cat "$STATE_FILE")
    
    stop_services
    apply_updates_and_promote_versions "$state_json"
    rm -f "$TRIGGER_FILE"
    start_services
    
    log "INFO: Update activation complete."
}

main "$@"
