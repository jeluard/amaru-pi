#!/bin/bash
set -euo pipefail

STATE_FILE="/home/pi/.amaru_update_state.json"
STAGING_DIR="/tmp"
LOCK_FILE="/tmp/amaru_check_update.lock"

declare -a BINARIES_TO_UPDATE=("amaru-pi")

declare -A GITHUB_REPOS
# GITHUB_REPOS["amaru"]="pragma-org/amaru"
GITHUB_REPOS["amaru-pi"]="jeluard/amaru-pi"
# GITHUB_REPOS["amaru-doctor"]="jeluard/amaru-doctor"

exec 200>"$LOCK_FILE"
flock -n 200 || { echo "ERROR: Another update process is running."; exit 1; }

log() { logger -t amaru-check "$1"; echo "$1"; }

abort() {
    log "ERROR: $1"
    exit 1
}

init_state_file() {
    if [ ! -f "$STATE_FILE" ]; then
        log "INFO: State file not found. Creating new one at ${STATE_FILE}"
        
        # Dynamically build the applications object
        local app_json=""
        for app in "${BINARIES_TO_UPDATE[@]}"; do
            app_json+="\"${app}\": {\"current_version\": \"v0.0.0\", \"pending_version\": \"\", \"staged_path\": \"\"},"
        done
        app_json=${app_json%,} # Remove trailing comma
        
        jq -n "{
            \"notify_after\": 0,
            \"applications\": { ${app_json} }
        }" > "$STATE_FILE"
        chown pi:pi "$STATE_FILE"
    fi
}

fetch_latest_release_json() {
    local repo="$1"
    local api_url="https://api.github.com/repos/${repo}/releases/latest"
    curl -s --fail --retry 3 --max-time 15 "$api_url" || return 1
}

extract_release_info() {
    local release_json="$1"
    local binary_name="$2"

    local latest_version
    latest_version=$(echo "$release_json" | jq -r '.tag_name')
    local download_url
    download_url=$(echo "$release_json" | jq -r ".assets[] | select(.name | contains(\"${binary_name}\") and contains(\"aarch64\")) | .browser_download_url")
    local checksum_url
    checksum_url=$(echo "$release_json" | jq -r ".assets[] | select(.name | endswith(\"checksums.txt\")) | .browser_download_url")

    [[ -n "$latest_version" && -n "$download_url" && -n "$checksum_url" ]] || return 1
    echo "${latest_version}|${download_url}|${checksum_url}"
}

verify_checksum() {
    local binary_name="$1"
    local archive="$2"
    local checksum_file="$3"

    local expected
    expected=$(grep "${binary_name}" "$checksum_file" | awk '{print $1}')
    local actual
    actual=$(sha256sum "$archive" | awk '{print $1}')

    [[ "$expected" == "$actual" ]]
}

stage_binary() {
    local binary_name="$1"
    local archive="$2"
    local staging_path="${STAGING_DIR}/${binary_name}.new"

    tar -xzf "$archive" -C "$STAGING_DIR"
    if [ ! -f "${STAGING_DIR}/${binary_name}" ]; then
        abort "Extracted file for ${binary_name} missing or misnamed."
    fi

    mv "${STAGING_DIR}/${binary_name}" "$staging_path"
    chmod +x "$staging_path"
    echo "$staging_path"
}

update_state_file_with_pending() {
    local binary_name="$1"
    local version="$2"
    local staged_path="$3"

    local tmp_state
    tmp_state=$(mktemp)
    jq \
      ".applications[\"${binary_name}\"].pending_version = \"${version}\" |
       .applications[\"${binary_name}\"].staged_path = \"${staged_path}\"" \
      "$STATE_FILE" > "$tmp_state"
    mv "$tmp_state" "$STATE_FILE"
    chown pi:pi "$STATE_FILE"
}

check_one_binary() {
    local binary_name="$1"
    local repo="${GITHUB_REPOS[$binary_name]}"

    log "INFO: Checking ${binary_name}..."

    local current_version
    current_version=$(jq -r ".applications[\"${binary_name}\"].current_version" "$STATE_FILE" 2>/dev/null || echo "v0.0.0")
    log "INFO: Current version: ${current_version}"

    local release_json
    release_json=$(fetch_latest_release_json "$repo") || { log "WARN: Failed to fetch release info."; return; }

    local info
    info=$(extract_release_info "$release_json" "$binary_name") || { log "WARN: Missing assets for ${binary_name}."; return; }

    IFS='|' read -r latest_version download_url checksum_url <<< "$info"
    log "INFO: Latest version: ${latest_version}"

    [[ "$current_version" == "$latest_version" ]] && { log "INFO: ${binary_name} is up to date."; return; }

    log "INFO: Downloading new assets..."
    local archive="/tmp/${binary_name}_latest.tar.gz"
    local checksum_file="/tmp/${binary_name}_checksums.txt"
    curl -sL -o "$archive" "$download_url"
    curl -sL -o "$checksum_file" "$checksum_url"

    verify_checksum "$binary_name" "$archive" "$checksum_file" || { abort "Checksum mismatch for ${binary_name}."; }

    log "INFO: Checksum verified."
    local staged
    staged=$(stage_binary "$binary_name" "$archive")
    rm -f "$archive" "$checksum_file"

    update_state_file_with_pending "$binary_name" "$latest_version" "$staged"
    log "SUCCESS: ${binary_name} staged at ${staged} (version ${latest_version})."
}

main() {
    init_state_file
    for binary in "${BINARIES_TO_UPDATE[@]}"; do
        echo "---"
        check_one_binary "$binary"
    done
    log "INFO: All update checks complete."
}

main "$@"
