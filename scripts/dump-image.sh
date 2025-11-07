#!/usr/bin/env bash
set -euo pipefail

OS="$(uname -s)"
IMAGE="sdcard.img"

list_devices() {
    if [[ "$OS" == "Darwin" ]]; then
        diskutil list | grep '^/dev/' | awk '{print $1}'
    else
        lsblk -dpno NAME | grep -E "/dev/(sd|mmcblk|nvme)"
    fi
}

# --- Step 1: Before ---
read -rp "➡️ Make sure your SD card is unplugged and press Enter to continue..."

BEFORE=$(list_devices)

read -rp "➡️ Now insert your SD card and press Enter to continue..."

echo "🔁  Waiting for the card detection.."
# Wait some time to ensure the sdcard is detected
sleep 3

# --- Step 2: After ---
AFTER=$(list_devices)

# --- Step 3: Detect new device ---
NEW_DEVICE=$(comm -13 <(echo "$BEFORE" | sort) <(echo "$AFTER" | sort) | head -n1 || true)

if [[ -z "${NEW_DEVICE}" ]]; then
    echo "⚠️ Could not auto-detect SD card device."
    read -rp "Please enter the device manually (e.g. /dev/sdb or /dev/disk3): " NEW_DEVICE
fi

echo "🧩 Detected device: ${NEW_DEVICE}"
read -rp "➡️ Confirm this is your SD card (yes/NO): " CONFIRM
if [[ "$CONFIRM" != "yes" ]]; then
    echo "❌ Aborted."
    exit 1
fi

echo "➡️ Sudo access is required, asking for credentials"


DEVICE="$NEW_DEVICE"

# --- Step 4: Perform dump ---
if [[ "$OS" == "Darwin" ]]; then
    RAW="${DEVICE/disk/rdisk}"
    echo "📀 Using raw device: ${RAW}"
    sudo dd if=${RAW} of=${IMAGE} bs=4m status=progress
else
    sudo dd if=${DEVICE} of=${IMAGE} bs=4M status=progress conv=sparse
fi
sudo chown "${USER}:$(id -gn)" ${IMAGE}

sync

echo "✅ Done! Dump saved as: ${IMAGE}"
