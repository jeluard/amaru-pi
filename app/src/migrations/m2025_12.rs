use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

use tracing::debug;
use tracing::error;
use tracing::info;

const HOTSPOT_SCRIPT: &str = include_str!("../../../overlays/usr/local/bin/amaru-hotspot.sh");
const HOTSPOT_SERVICE: &str =
    include_str!("../../../overlays/etc/systemd/system/amaru-hotspot.service");
const HOTSPOT_TIMER: &str =
    include_str!("../../../overlays/etc/systemd/system/amaru-hotspot.timer");

const UPDATER_SCRIPT: &str = r#"#!/bin/bash
set -euo pipefail

if [ -f /home/pi/amaru.env ]; then
    set -a
    source /home/pi/amaru.env
    set +a
fi

STATE_FILE="/home/pi/.amaru_update_state.json"
STAGING_DIR="/tmp"
LOCK_FILE="/tmp/amaru_check_update.lock"

declare -a BINARIES_TO_UPDATE=("amaru-pi" "amaru" "amaru-doctor")

log() { logger -t amaru-check "$1"; echo "$1" >&2; }

exec 200>"$LOCK_FILE"
flock -n 200 || { echo "ERROR: Another update process is running."; exit 1; }

declare -A GITHUB_REPOS
GITHUB_REPOS["amaru"]="pragma-org/amaru"
GITHUB_REPOS["amaru-pi"]="jeluard/amaru-pi"
GITHUB_REPOS["amaru-doctor"]="jeluard/amaru-doctor"

if [ -n "${AMARU_REPO_OVERRIDE:-}" ]; then
    log "INFO: Overriding amaru repo to ${AMARU_REPO_OVERRIDE}"
    GITHUB_REPOS["amaru"]="${AMARU_REPO_OVERRIDE}"
fi

if [ -n "${AMARU_PI_REPO_OVERRIDE:-}" ]; then
    log "INFO: Overriding amaru-pi repo to ${AMARU_PI_REPO_OVERRIDE}"
    GITHUB_REPOS["amaru-pi"]="${AMARU_PI_REPO_OVERRIDE}"
fi

if [ -n "${AMARU_DOCTOR_REPO_OVERRIDE:-}" ]; then
    log "INFO: Overriding amaru-doctor repo to ${AMARU_DOCTOR_REPO_OVERRIDE}"
    GITHUB_REPOS["amaru-doctor"]="${AMARU_DOCTOR_REPO_OVERRIDE}"
fi

abort() {
    log "ERROR: $1"
    exit 1
}

init_state_file() {
    if [ ! -f "$STATE_FILE" ]; then
        log "INFO: Creating state file..."
        jq -n '{
            "notify_after": 0,
            "applications": {
                "amaru-pi": { "current_version": "v0.0.0", "current_source": "", "pending_version": "", "pending_source": "", "staged_path": "" },
                "amaru": { "current_version": "v0.0.0", "current_source": "", "pending_version": "", "pending_source": "", "staged_path": "" },
                "amaru-doctor": { "current_version": "v0.0.0", "current_source": "", "pending_version": "", "pending_source": "", "staged_path": "" }
            }
        }' > "$STATE_FILE"
        chown pi:pi "$STATE_FILE"
    fi
}

fetch_latest_release_json() {
    local repo="$1"
    local url="https://api.github.com/repos/${repo}/releases/latest"
    
    log "DEBUG: Fetching latest release from $url"
    
    local http_code
    http_code=$(curl -s -w "%{http_code}" -o /tmp/amaru_curl_body "$url")
    
    if [ "$http_code" -eq 200 ]; then
        log "DEBUG: Fetch successful (HTTP 200)"
        cat /tmp/amaru_curl_body
        rm -f /tmp/amaru_curl_body
        return 0
    elif [ "$http_code" -eq 404 ]; then
        log "WARN: Repo $repo exists but has no 'latest' release (HTTP 404)"
        rm -f /tmp/amaru_curl_body
        return 2
    else
        log "ERROR: Failed to fetch release from $repo. HTTP Code: $http_code"
        rm -f /tmp/amaru_curl_body
        return 1
    fi
}

extract_release_info() {
    local release_json="$1"
    local binary_name="$2"
    
    if ! echo "$release_json" | jq empty >/dev/null 2>&1; then
        log "ERROR: Invalid JSON received for $binary_name"
        return 1
    fi

    local latest_version=$(echo "$release_json" | jq -r '.tag_name // empty')
    
    local download_url=$(echo "$release_json" | jq -r '.assets[] | select(.name | contains("linux") and contains("aarch64") and contains(".tar.gz")) | .browser_download_url' | head -n 1)
    
    if [ -z "$latest_version" ] || [ -z "$download_url" ] || [ "$download_url" == "null" ]; then
        log "DEBUG: No suitable assets found in release '$latest_version' for $binary_name"
        return 1
    fi
    
    echo "${latest_version}|${download_url}"
}

stage_binary() {
    local binary_name="$1"
    local archive="$2"
    local staging_path="${STAGING_DIR}/${binary_name}.new"
    local extract_dir="${STAGING_DIR}/${binary_name}_extract"

    rm -rf "$extract_dir"
    mkdir -p "$extract_dir"
    
    log "DEBUG: Extracting $archive to $extract_dir"
    tar -xzf "$archive" -C "$extract_dir"
    local extracted_bin=$(find "$extract_dir" -type f -name "${binary_name}" | head -n 1)

    if [ -z "$extracted_bin" ]; then
        abort "Extracted file for ${binary_name} missing in $extract_dir."
    fi

    log "DEBUG: Moving $extracted_bin to $staging_path"
    mv "$extracted_bin" "$staging_path"
    chmod +x "$staging_path"
    echo "$staging_path"
}

update_state_file() {
    local binary="$1"
    local ver="$2"
    local path="$3"
    local src="$4"
    
    local tmp=$(mktemp)
    jq ".applications[\"${binary}\"].pending_version = \"${ver}\" | \
        .applications[\"${binary}\"].pending_source = \"${src}\" | \
        .applications[\"${binary}\"].staged_path = \"${path}\"" \
        "$STATE_FILE" > "$tmp"
    mv "$tmp" "$STATE_FILE"
    chown pi:pi "$STATE_FILE"
}

check_one_binary() {
    local binary="$1"
    local target_repo="${GITHUB_REPOS[$binary]}"

    log "INFO: Checking ${binary} against ${target_repo}..."
    
    if ! jq -e ".applications[\"${binary}\"]" "$STATE_FILE" > /dev/null; then
         local tmp=$(mktemp)
         jq ".applications[\"${binary}\"] = { \"current_version\": \"v0.0.0\", \"current_source\": \"\", \"pending_version\": \"\", \"pending_source\": \"\", \"staged_path\": \"\" }" "$STATE_FILE" > "$tmp"
         mv "$tmp" "$STATE_FILE"
         chown pi:pi "$STATE_FILE"
    fi

    local current_version=$(jq -r ".applications[\"${binary}\"].current_version // \"v0.0.0\"" "$STATE_FILE")
    local current_source=$(jq -r ".applications[\"${binary}\"].current_source // \"\"" "$STATE_FILE")
    
    local release_json
    if ! release_json=$(fetch_latest_release_json "$target_repo"); then
        local rc=$?
        if [ $rc -eq 2 ]; then
            log "WARN: No release found for ${target_repo} (404). Skipping."
        else
            log "WARN: Failed to fetch releases for ${target_repo}. Skipping."
        fi
        return
    fi
    
    local info
    if ! info=$(extract_release_info "$release_json" "$binary"); then
        log "WARN: Valid release found, but no matching aarch64 assets for ${binary}. Skipping."
        return
    fi
    
    IFS='|' read -r latest_version download_url <<< "$info"

    if [[ "$current_version" != "$latest_version" ]] || [[ "$current_source" != "$target_repo" ]]; then
        log "INFO: Found update ${latest_version} from ${target_repo} (Current: ${current_version} from ${current_source})"
        
        local archive="/tmp/${binary}_latest.tar.gz"
        log "INFO: Downloading $download_url to $archive"
        curl -sL -o "$archive" "$download_url"
        
        local staged=$(stage_binary "$binary" "$archive")
        rm -f "$archive"
        
        update_state_file "$binary" "$latest_version" "$staged" "$target_repo"
        log "SUCCESS: Staged ${binary}"
    else
        log "INFO: ${binary} is up to date"
    fi
}

main() {
    init_state_file
    for binary in "${BINARIES_TO_UPDATE[@]}"; do
        check_one_binary "$binary"
    done
}

main "$@"
"#;

const ACTIVATE_SCRIPT: &str = r#"#!/bin/bash
set -euo pipefail

STATE_FILE="/home/pi/.amaru_update_state.json"
BIN_DIR="/home/pi/bin"
TRIGGER_FILE="/home/pi/.update_requested"
LOCK_FILE="/tmp/amaru_update.lock"

declare -a STOP_UNITS=("amaru-pi.service" "getty@tty1.service" "amaru.service")
declare -a START_UNITS=("getty@tty1.service" "amaru.service")

exec 200>"$LOCK_FILE"
flock -n 200 || { echo "ERROR: Another update is in progress."; exit 1; }

log() { logger -t amaru-update "$1"; echo "$1"; }

apply_updates() {
    local state_json=$(cat "$STATE_FILE")
    local new_state_json="$state_json"

    # Stop services
    for unit in "${STOP_UNITS[@]}"; do
        systemctl stop "$unit" || true
    done

    for app_name in $(echo "$state_json" | jq -r '.applications | keys[]'); do
        local pending_ver=$(echo "$state_json" | jq -r ".applications[\"${app_name}\"].pending_version")
        local pending_src=$(echo "$state_json" | jq -r ".applications[\"${app_name}\"].pending_source // \"\"")
        local staged=$(echo "$state_json" | jq -r ".applications[\"${app_name}\"].staged_path")

        if [ -n "$pending_ver" ] && [ -f "$staged" ]; then
            log "INFO: Updating ${app_name} to ${pending_ver}..."
            
            # Backup
            [ -f "${BIN_DIR}/${app_name}" ] && mv "${BIN_DIR}/${app_name}" "${BIN_DIR}/${app_name}.bak"
            
            # Install
            mv "$staged" "${BIN_DIR}/${app_name}"
            chmod +x "${BIN_DIR}/${app_name}"

            # Clear pending state and promote Source Repo
             new_state_json=$(echo "$new_state_json" | jq \
                ".applications[\"${app_name}\"].current_version = \"${pending_ver}\" |
                 .applications[\"${app_name}\"].current_source = \"${pending_src}\" |
                 .applications[\"${app_name}\"].pending_version = \"\" |
                 .applications[\"${app_name}\"].pending_source = \"\" |
                 .applications[\"${app_name}\"].staged_path = \"\"")
        fi
    done

    # Save state
    echo "$new_state_json" | jq '.notify_after = 0' > "$STATE_FILE"
    chown pi:pi "$STATE_FILE"

    # Start services
    for unit in "${START_UNITS[@]}"; do
        systemctl start "$unit" || log "ERROR: Failed to start $unit"
    done
}

main() {
    apply_updates
    rm -f "$TRIGGER_FILE"
}

main "$@"
"#;

const START_AMARU_SCRIPT: &str = r#"#!/bin/bash
set -euo pipefail

# This wrapper detects if the installed amaru binary supports 'run' or 'daemon'

BIN="/home/pi/bin/amaru"

if [ ! -f "$BIN" ]; then
    echo "ERROR: $BIN not found"
    exit 1
fi

# Check help output to see which command is supported
# Older binaries have 'daemon', new ones have 'run'.
if "$BIN" --help 2>&1 | grep -q "daemon"; then
    exec "$BIN" daemon
else
    exec "$BIN" run
fi
"#;

fn write_file(path: &str, content: &str, mode: u32) -> io::Result<()> {
    debug!("Writing to {}", path);
    let p = Path::new(path);
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(p, content)?;
    let metadata = fs::metadata(p)?;
    let mut perms = metadata.permissions();
    perms.set_mode(mode);
    fs::set_permissions(p, perms)?;
    Ok(())
}

fn write_script(path: &str, content: &str) -> io::Result<()> {
    write_file(path, content, 0o755)
}

fn run_command_checked(program: &str, args: &[&str]) -> anyhow::Result<()> {
    let status = Command::new(program).args(args).status()?;
    if !status.success() {
        return Err(anyhow::anyhow!(
            "{} {:?} failed with status {}",
            program,
            args,
            status
        ));
    }

    Ok(())
}

fn patch_amaru_service() -> anyhow::Result<()> {
    let service_path = "/etc/systemd/system/amaru.service";
    let path = Path::new(service_path);

    if !path.exists() {
        error!("{} doesn't exist", service_path);
        return Ok(());
    }

    let content = fs::read_to_string(path)?;
    if content.contains("ExecStart=/home/pi/scripts/start-amaru.sh") {
        debug!("amaru.service already uses wrapper");
        return Ok(());
    }

    info!("Patching amaru.service to use wrapper...");

    let new_lines: Vec<String> = content
        .lines()
        .map(|line| {
            if line.trim().starts_with("ExecStart=") {
                "ExecStart=/home/pi/scripts/start-amaru.sh".to_string()
            } else {
                line.to_string()
            }
        })
        .collect();

    let new_content = new_lines.join("\n");
    fs::write(path, new_content)?;
    Command::new("systemctl").arg("daemon-reload").status()?;

    debug!("amaru.service patched and reloaded.");
    Ok(())
}

pub fn run() -> anyhow::Result<()> {
    debug!("Checking scripts...");

    write_script("/home/pi/scripts/updater.sh", UPDATER_SCRIPT)?;
    write_script("/home/pi/scripts/activate-update.sh", ACTIVATE_SCRIPT)?;
    write_script("/home/pi/scripts/start-amaru.sh", START_AMARU_SCRIPT)?;
    write_script("/usr/local/bin/amaru-hotspot.sh", HOTSPOT_SCRIPT)?;
    write_file(
        "/etc/systemd/system/amaru-hotspot.service",
        HOTSPOT_SERVICE,
        0o644,
    )?;
    write_file(
        "/etc/systemd/system/amaru-hotspot.timer",
        HOTSPOT_TIMER,
        0o644,
    )?;
    patch_amaru_service()?;

    run_command_checked("/usr/local/bin/amaru-hotspot.sh", &["ensure"])?;
    run_command_checked("systemctl", &["daemon-reload"])?;
    run_command_checked("systemctl", &["enable", "--now", "amaru-hotspot.timer"])?;

    Ok(())
}
