#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

source ${SCRIPT_DIR}/helper.sh

usage() {
  echo "Usage: $0 [envs|wifi|all]"
  echo "  envs  - configure only environment variables"
  echo "  wifi  - configure only WiFi settings"
  echo "  all   - (default) configure everything"
  exit 1
}

configure_envs() {
  echo "🔧 Configuring environment variables..."
  "${SCRIPT_DIR}/configure/set-envs.sh"
}

configure_wifi() {
  echo "📶 Configuring WiFi..."
  "${SCRIPT_DIR}/configure/set-wifi-conf.sh"
}

configure_all() {
  configure_envs
  configure_wifi
}

ACTION="${1:-all}"

case "$ACTION" in
  envs)
    configure_envs
    ;;
  wifi)
    configure_wifi
    ;;
  all)
    configure_all
    ;;
  *)
    usage
    ;;
esac

echo "✅ Configuration complete."

ask_and_reboot