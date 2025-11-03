#!/usr/bin/env bash

# A simple script to remotely execute configure-pi.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

source ${SCRIPT_DIR}/helper/remote.sh

usage() {
  echo "Usage: $0 user@host [envs|wifi|all]"
  echo
  echo "Examples:"
  echo "  $0 pi@raspberrypi.local          # Run full configuration (default: all)"
  echo "  $0 pi@raspberrypi.local envs     # Only configure environment variables"
  echo "  $0 pi@raspberrypi.local wifi     # Only configure WiFi"
  echo
  echo "Environment variables required:"
  echo "  AMARU_WORDS          (for envs or all)"
  echo "  AMARU_WIFI_SSID      (for wifi or all)"
  echo "  AMARU_WIFI_PASSWORD  (for wifi or all)"
  exit 2
}

validate_action() {
  local action="$1"
  case "$action" in
    envs|wifi|all) ;;
    *) 
      echo "❌ Invalid action: '$action'"
      usage
      ;;
  esac
}

if [[ -n "${SSH_REMOTE:-}" ]]; then
  ACTION="${1:-all}"
else
  SSH_REMOTE="${1:-}"
  ACTION="${2:-all}"
  validate_action "$ACTION"
fi

if [[ "$ACTION" == "envs" || "$ACTION" == "all" ]]; then
  if [[ -z "${AMARU_WORDS:-}" ]]; then
    echo "Error: AMARU_WORDS is not set (required for envs)."
    exit 1
  fi
fi

if [[ "$ACTION" == "wifi" || "$ACTION" == "all" ]]; then
  if [[ -z "${AMARU_WIFI_SSID:-}" ]]; then
    echo "Error: AMARU_WIFI_SSID is not set (required for WiFi)."
    exit 1
  fi

  if [[ -z "${AMARU_WIFI_PASSWORD:-}" ]]; then
    echo "Error: AMARU_WIFI_PASSWORD is not set (required for WiFi)."
    exit 1
  fi
fi

echo "🚀 Running remote configuration on ${SSH_REMOTE} (${ACTION})..."

env_vars=()

case "$ACTION" in
  envs)
    env_vars+=(AMARU_WORDS)
    ;;
  wifi)
    env_vars+=(AMARU_WIFI_SSID AMARU_WIFI_PASSWORD)
    ;;
  all)
    env_vars+=(AMARU_WORDS AMARU_WIFI_SSID AMARU_WIFI_PASSWORD)
    ;;
esac

run_remote_script \
  "${SSH_REMOTE}" \
  "${SSH_OPTS:-}" \
  "/home/pi/scripts/configure.sh" \
  "${env_vars[@]}" \
  "${ACTION}"

rc=$?

if [[ $rc -ne 0 ]]; then
    echo "❌ Remote configuration failed with code $rc"
    exit $rc
fi

echo "✅ Remote configuration succeeded (${ACTION})"
