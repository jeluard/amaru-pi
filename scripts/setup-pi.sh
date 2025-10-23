#!/usr/bin/env bash
set -euo pipefail

# Usage: ./deploy-overlays.sh user@host
# Optional env vars:
#   SSH_OPTS="-p 2222"
#   DEPLOY_ACTION=reboot|restart|none
#
# Must run from a directory containing "overlays/"

REMOTE="${1:-}"
SSH_OPTS="${SSH_OPTS:-}"
DEPLOY_ACTION="${DEPLOY_ACTION:-}"
OVERLAYS_DIR="overlays"
SETUP_SCRIPT="/home/pi/setup.sh"

if [[ -z "$REMOTE" ]]; then
  echo "Usage: $0 user@host"
  exit 2
fi

[[ -d "$OVERLAYS_DIR" ]] || { echo "Error: '$OVERLAYS_DIR' directory not found."; exit 1; }

for cmd in ssh tar sha256sum; do
  command -v "$cmd" >/dev/null || { echo "Error: '$cmd' required."; exit 1; }
done

echo "→ Scanning overlays for executables and valid systemd services..."

exe_targets=()
service_names=()

# Detect executables and services
while IFS= read -r -d '' f; do
  rel="${f#${OVERLAYS_DIR}/}"
  remote_path="/${rel}"

  if [[ "$f" == overlays/etc/systemd/system/*.service ]]; then
    service_names+=("$(basename "$f")")
  fi

  if [[ -f "$f" ]]; then
    make_exec=false
    if [[ -x "$f" ]]; then
      make_exec=true
    elif command -v file >/dev/null; then
      file_out="$(file -b "$f" 2>/dev/null || true)"
      if echo "$file_out" | grep -Eiq 'ELF|executable|PE32|Mach-O'; then
        make_exec=true
      elif head -n 1 "$f" | grep -q '^#!'; then
        make_exec=true
      fi
    fi
    $make_exec && exe_targets+=("$remote_path")
  fi
done < <(find "${OVERLAYS_DIR}" -type f -print0)

echo "→ Files to make executable: ${#exe_targets[@]}"
echo "→ Systemd services to enable: ${#service_names[@]}"

if [[ -z "$DEPLOY_ACTION" ]]; then
  read -rp "Proceed with deployment to ${REMOTE}? [y/N] " ok
  [[ "${ok,,}" == "y" ]] || { echo "Aborted."; exit 0; }
else
  echo "DEPLOY_ACTION set → non-interactive deploy"
fi

echo "→ Uploading overlays/ → ${REMOTE}:/ ..."
(
  cd "${OVERLAYS_DIR}"
  tar -cf - . | ssh $SSH_OPTS "$REMOTE" 'sudo tar --no-same-owner -C /  -xpf - ; sudo chown -R $(id -u):$(id -g) $HOME'
)
echo "✓ Upload complete."

if [[ ${#exe_targets[@]} -gt 0 ]]; then
  echo "→ Setting executable bits on remote..."
  printf '%s\0' "${exe_targets[@]}" | ssh $SSH_OPTS "$REMOTE" 'sudo bash -s' <<'REMOTE_CHMOD'
while IFS= read -r -d '' path; do
  [[ -e "$path" ]] || { echo "⚠️  Missing $path"; continue; }
  echo "chmod +x $path"
  sudo chmod +x -- "$path" || echo "⚠️  chmod failed for $path" >&2
done
REMOTE_CHMOD
fi

# --- Systemd service sync ---
if [[ ${#service_names[@]} -gt 0 ]]; then
  echo "→ Synchronizing systemd services..."
  printf '%s\n' "${service_names[@]}" | ssh $SSH_OPTS "$REMOTE" 'sudo bash -s' <<'REMOTE_SYNC'
set -euo pipefail
overlay_services=()
while read -r svc; do
  [[ -n "$svc" ]] && overlay_services+=("$svc")
done

enabled_services=($(systemctl list-unit-files --type=service --state=enabled --no-legend | awk '{print $1}' || true))

for svc in "${overlay_services[@]}"; do
  if ! systemctl is-enabled --quiet "$svc"; then
    echo "Enabling $svc"
    sudo systemctl enable "$svc" || echo "⚠️  enable failed for $svc" >&2
  fi
done

for svc in "${enabled_services[@]}"; do
  if [[ "$svc" == *".service" ]] && [[ -f "/etc/systemd/system/$svc" ]]; then
    found=false
    for o in "${overlay_services[@]}"; do
      if [[ "$o" == "$svc" ]]; then
        found=true
        break
      fi
    done
    if ! $found; then
      echo "Disabling obsolete $svc"
      sudo systemctl disable "$svc" || echo "⚠️  disable failed for $svc" >&2
    fi
  fi
done
REMOTE_SYNC
else
  echo "No valid services under overlays/etc/systemd/system/. Skipping systemd sync."
fi

# ──────────────────────────────────────────────────────────────
# Optional setup script execution ($SETUP_SCRIPT)
# ──────────────────────────────────────────────────────────────
if [[ -n "$RUN_SETUP" ]]; then
  echo "→ Running remote setup script: $SETUP_SCRIPT"
  ssh $SSH_OPTS "$REMOTE" 'if [[ -f $SETUP_SCRIPT ]]; then sudo bash $SETUP_SCRIPT; else echo "⚠️  $SETUP_SCRIPT not found"; fi'
else
  echo "Skipping $SETUP_SCRIPT (RUN_SETUP not set)"
fi

# --- Final action ---
run_action() {
  local action="$1"
  case "$action" in
    reboot)
      echo "→ Rebooting remote..."
      ssh $SSH_OPTS "$REMOTE" 'sudo systemctl reboot'
      ;;
    restart)
      if [[ ${#service_names[@]} -eq 0 ]]; then
        echo "No overlay services found."
        return
      fi
      echo "→ Restarting or starting overlay services..."
      printf '%s\0' "${service_names[@]}" | ssh $SSH_OPTS "$REMOTE" 'sudo bash -s' <<'REMOTE_RESTART_START'
while IFS= read -r -d '' svc; do
  if sudo systemctl is-active --quiet "$svc"; then
    echo "Restarting $svc"
    sudo systemctl restart "$svc" || echo "⚠️  restart failed for $svc" >&2
  else
    echo "Starting $svc"
    sudo systemctl start "$svc" || echo "⚠️  start failed for $svc" >&2
  fi
done
REMOTE_RESTART_START
      ;;
    none)
      echo "No post-deploy action."
      ;;
    *)
      echo "Unknown DEPLOY_ACTION='$action'" >&2
      exit 1
      ;;
  esac
}

if [[ -n "$DEPLOY_ACTION" ]]; then
  run_action "$DEPLOY_ACTION"
else
  echo
  echo "Choose post-deploy action:"
  PS3="Select an option (1-3): "
  options=("Reboot remote machine" "Restart/start all overlay services" "Do nothing")
  select opt in "${options[@]}"; do
    case "$REPLY" in
      1) run_action reboot; break ;;
      2) run_action restart; break ;;
      3) run_action none; break ;;
      *) echo "Invalid selection." ;;
    esac
  done
fi

echo "✓ Done."
