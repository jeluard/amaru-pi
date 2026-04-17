#!/bin/bash
set -euo pipefail

WIFI_INTERFACE="${AMARU_WIFI_INTERFACE:-wlan0}"
HOTSPOT_CONNECTION="${AMARU_HOTSPOT_CONNECTION:-amaru-hotspot}"
HOTSPOT_SSID="${AMARU_HOTSPOT_SSID:-Amaru Setup}"
HOTSPOT_PASSWORD="${AMARU_HOTSPOT_PASSWORD:-amaru-setup}"

if [[ ${#HOTSPOT_PASSWORD} -lt 8 ]]; then
    echo "❌ AMARU_HOTSPOT_PASSWORD must be at least 8 characters long"
    exit 1
fi

echo "[1/5] Install NetworkManager hotspot dependencies"
sudo apt-get -qq update
sudo DEBIAN_FRONTEND=noninteractive apt-get install -y network-manager dnsmasq-base

echo "[2/5] Enable NetworkManager and retire the legacy hotspot stack"
sudo systemctl enable NetworkManager >/dev/null 2>&1 || true
if ! sudo systemctl is-active --quiet NetworkManager; then
    sudo systemctl start NetworkManager
fi

sudo systemctl disable --now hotspot-nat.service 2>/dev/null || true
sudo systemctl disable --now hostapd.service 2>/dev/null || true
sudo systemctl disable --now dnsmasq.service 2>/dev/null || true
sudo pkill -x hostapd 2>/dev/null || true
sudo rm -f /etc/dnsmasq.d/hotspot.conf
sudo rm -f /etc/hostapd/hostapd_ap.conf
sudo rm -f /etc/network/if-up.d/iptables
sudo rm -f /etc/iptables.ipv4.nat

echo "[3/5] Make sure Wi-Fi is enabled"
sudo nmcli radio wifi on

echo "[4/5] Create or update the fallback hotspot profile"
sudo /usr/local/bin/amaru-hotspot.sh ensure

echo "[5/5] Fallback hotspot profile ready"
echo "Configured NetworkManager hotspot profile '$HOTSPOT_CONNECTION' for SSID '$HOTSPOT_SSID'."
echo "systemd will activate it when no upstream Wi-Fi connection is active."
