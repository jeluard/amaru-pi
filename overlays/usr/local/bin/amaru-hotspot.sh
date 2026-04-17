#!/bin/bash
set -euo pipefail

STATE_DIR="/run/amaru"
LOCK_FILE="$STATE_DIR/hotspot.lock"
STATE_FILE="$STATE_DIR/hotspot.offline_since"
ENV_FILE="/home/pi/amaru.env"

mkdir -p "$STATE_DIR"
exec 9>"$LOCK_FILE"
flock -n 9 || exit 0

if [[ -f "$ENV_FILE" ]]; then
    set -a
    # shellcheck disable=SC1091
    source "$ENV_FILE"
    set +a
fi

WIFI_INTERFACE="${AMARU_WIFI_INTERFACE:-wlan0}"
HOTSPOT_CONNECTION="${AMARU_HOTSPOT_CONNECTION:-amaru-hotspot}"
HOTSPOT_SSID="${AMARU_HOTSPOT_SSID:-Amaru Setup}"
HOTSPOT_PASSWORD="${AMARU_HOTSPOT_PASSWORD:-amaru-setup}"
OFFLINE_GRACE_SECS="${AMARU_HOTSPOT_OFFLINE_GRACE_SECS:-20}"

trim() {
    sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//'
}

log() {
    local message="$1"
    echo "$message"
    logger -t amaru-hotspot "$message"
}

clear_offline_since() {
    rm -f "$STATE_FILE"
}

remember_offline_since() {
    if [[ ! -f "$STATE_FILE" ]]; then
        date +%s >"$STATE_FILE"
    fi

    cat "$STATE_FILE"
}

has_wifi_interface() {
    nmcli -g DEVICE device status 2>/dev/null | grep -Fxq "$WIFI_INTERFACE"
}

device_state() {
    local raw
    raw="$(nmcli -g GENERAL.STATE device show "$WIFI_INTERFACE" 2>/dev/null | head -n 1 || true)"
    raw="$(printf '%s' "$raw" | trim)"

    if [[ -z "$raw" ]]; then
        return 1
    fi

    printf '%s\n' "${raw%% *}"
}

current_connection_name() {
    local raw
    raw="$(nmcli -g GENERAL.CONNECTION device show "$WIFI_INTERFACE" 2>/dev/null | head -n 1 || true)"
    raw="$(printf '%s' "$raw" | trim)"

    if [[ -z "$raw" || "$raw" == "--" ]]; then
        return 1
    fi

    printf '%s\n' "$raw"
}

ensure_profile() {
    if [[ ${#HOTSPOT_PASSWORD} -lt 8 ]]; then
        echo "AMARU_HOTSPOT_PASSWORD must be at least 8 characters long" >&2
        exit 1
    fi

    nmcli radio wifi on >/dev/null

    if ! nmcli -g NAME connection show "$HOTSPOT_CONNECTION" >/dev/null 2>&1; then
        nmcli connection add \
            type wifi \
            ifname "$WIFI_INTERFACE" \
            con-name "$HOTSPOT_CONNECTION" \
            autoconnect no \
            ssid "$HOTSPOT_SSID" >/dev/null
    fi

    nmcli connection modify "$HOTSPOT_CONNECTION" \
        connection.autoconnect no \
        connection.interface-name "$WIFI_INTERFACE" \
        802-11-wireless.mode ap \
        802-11-wireless.band bg \
        802-11-wireless.ssid "$HOTSPOT_SSID" \
        ipv4.method shared \
        ipv6.method ignore \
        wifi-sec.key-mgmt wpa-psk \
        wifi-sec.psk "$HOTSPOT_PASSWORD" >/dev/null
}

stop_hotspot() {
    nmcli connection down "$HOTSPOT_CONNECTION" >/dev/null 2>&1 || true
    clear_offline_since
}

start_hotspot() {
    ensure_profile
    nmcli device disconnect "$WIFI_INTERFACE" >/dev/null 2>&1 || true
    nmcli connection up "$HOTSPOT_CONNECTION" >/dev/null
    clear_offline_since
}

reconcile() {
    local state connection_name now offline_since

    if ! has_wifi_interface; then
        clear_offline_since
        exit 0
    fi

    state="$(device_state || true)"
    connection_name="$(current_connection_name || true)"

    if [[ -z "$state" ]]; then
        clear_offline_since
        exit 0
    fi

    if [[ "$connection_name" == "$HOTSPOT_CONNECTION" ]]; then
        clear_offline_since
        exit 0
    fi

    if [[ "$state" =~ ^(40|50|60|70|80|90)$ ]]; then
        stop_hotspot
        exit 0
    fi

    if [[ "$state" == "100" && -n "$connection_name" ]]; then
        stop_hotspot
        exit 0
    fi

    if [[ "$state" != "20" && "$state" != "30" && "$state" != "120" ]]; then
        clear_offline_since
        exit 0
    fi

    now="$(date +%s)"
    offline_since="$(remember_offline_since)"
    if (( now - offline_since < OFFLINE_GRACE_SECS )); then
        exit 0
    fi

    log "Starting fallback hotspot profile '$HOTSPOT_CONNECTION'"
    start_hotspot
}

case "${1:-reconcile}" in
    ensure)
        ensure_profile
        ;;
    up)
        start_hotspot
        ;;
    down)
        stop_hotspot
        ;;
    reconcile)
        reconcile
        ;;
    *)
        echo "Usage: $0 {ensure|up|down|reconcile}" >&2
        exit 1
        ;;
esac