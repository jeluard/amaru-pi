#!/bin/bash

set -euo pipefail

STATE_FILE="/home/pi/.amaru_update_state.json"
STAGING_DIR="/tmp"

declare -a BINARIES_TO_UPDATE=("amaru-pi")

declare -A GITHUB_REPOS
GITHUB_REPOS["amaru-pi"]="jeluard/amaru-pi"

if [ ! -f "$STATE_FILE" ]; then
    echo "INFO: State file not found. Creating a new one at ${STATE_FILE}"
    jq -n '{
        "notify_after": 0,
        "applications": {
            "amaru": {"current_version": "v0.0.0", "update_available": false, "staged_path": ""},
            "amaru-pi": {"current_version": "v0.0.0", "update_available": false, "staged_path": ""},
            "amaru-doctor": {"current_version": "v0.0.0", "update_available": false, "staged_path": ""}
        }
    }' > "$STATE_FILE"
fi

function check_one_binary() {
    local binary_name="$1"
    local github_repo="${GITHUB_REPOS[$binary_name]}"
    local staging_path="${STAGING_DIR}/${binary_name}.new"

    echo "INFO: Processing check for ${binary_name}..."

    local current_version
    current_version=$(jq -r ".applications[\"${binary_name}\"].current_version" "$STATE_FILE")
    echo "INFO: Current local version is ${current_version}"

    local api_url="https://api.github.com/repos/${github_repo}/releases/latest"
    local latest_release_json
    latest_release_json=$(curl -s -f "${api_url}")

    if [ -z "$latest_release_json" ]; then
        echo "WARN: Could not fetch release info for ${binary_name}. Skipping."
        return
    fi

    local latest_version
    latest_version=$(echo "${latest_release_json}" | jq -r '.tag_name')
    echo "INFO: Latest available version is ${latest_version}"

    if [ "${current_version}" == "${latest_version}" ]; then
        echo "INFO: ${binary_name} is already up to date."
        return
    fi

    local download_url
    download_url=$(echo "${latest_release_json}" | jq -r ".assets[] | select(.name | contains(\"${binary_name}\") and contains(\"aarch64\")) | .browser_download_url")
    local checksum_url
    checksum_url=$(echo "${latest_release_json}" | jq -r ".assets[] | select(.name | endswith(\"checksums.txt\")) | .browser_download_url")

    if [ -z "$download_url" ] || [ -z "$checksum_url" ]; then
        echo "ERROR: Could not find required assets (binary or checksum) for ${binary_name} in release ${latest_version}. Aborting."
        return
    fi

    echo "INFO: Downloading assets for new version..."
    local temp_checksum_file="/tmp/${binary_name}_checksums.txt"
    local temp_archive="/tmp/${binary_name}_latest.tar.gz"
    curl -sL -o "${temp_checksum_file}" "${checksum_url}"
    curl -sL -o "${temp_archive}" "${download_url}"

    echo "INFO: Verifying checksum..."
    local expected_checksum
    expected_checksum=$(grep "${binary_name}" "${temp_checksum_file}" | awk '{print $1}')
    local actual_checksum
    actual_checksum=$(sha256sum "${temp_archive}" | awk '{print $1}')

    if [ "${expected_checksum}" != "${actual_checksum}" ]; then
        echo "ERROR: Checksum mismatch for ${binary_name}! Downloaded file is corrupt. Aborting."
        rm "${temp_archive}" "${temp_checksum_file}"
        return
    fi
    echo "INFO: Checksum OK."

    echo "INFO: Extracting to staging location: ${staging_path}"
    tar -xzf "${temp_archive}" -C "${STAGING_DIR}"
    mv "${STAGING_DIR}/${binary_name}" "${staging_path}"
    
    rm "${temp_archive}" "${temp_checksum_file}"

    echo "INFO: Staging complete. Updating state file."
    
    local temp_state_file
    temp_state_file=$(mktemp)
    
    jq \
      ".applications[\"${binary_name}\"].update_available = true | \
       .applications[\"${binary_name}\"].staged_path = \"${staging_path}\" | \
       .applications[\"${binary_name}\"].current_version = \"${latest_version}\"" \
      "$STATE_FILE" > "$temp_state_file"

    mv "$temp_state_file" "$STATE_FILE"

    echo "SUCCESS: ${binary_name} updated to ${latest_version} and staged."
}

for binary in "${BINARIES_TO_UPDATE[@]}"; do
    echo "---"
    check_one_binary "${binary}"
done

echo "---"
echo "All update checks are complete."
