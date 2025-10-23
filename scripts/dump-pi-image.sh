#!/usr/bin/env bash
set -euo pipefail

OS="$(uname -s)"
DATE="$(date +%Y%m%d_%H%M%S)"
OUT="sdcard_${DATE}.img"

list_devices() {
    if [[ "$OS" == "Darwin" ]]; then
        diskutil list | grep '^/dev/' | awk '{print $1}'
    else
        lsblk -dpno NAME | grep -E "/dev/(sd|mmcblk|nvme)"
    fi
}

# --- Step 1: Before ---
read -rp "🔁 Make sure your SD card is unplugged and press Enter to continue..."

BEFORE=$(list_devices)

read -rp "🔁 Now insert your SD card, wait a couple seconds and press Enter to continue..."

# --- Step 2: After ---
AFTER=$(list_devices)

# --- Step 3: Detect new device ---
NEW_DEVICE=$(comm -13 <(echo "$BEFORE" | sort) <(echo "$AFTER" | sort) | head -n1 || true)

if [[ -z "$NEW_DEVICE" ]]; then
    echo "⚠️ Could not auto-detect SD card device."
    read -rp "Please enter the device manually (e.g. /dev/sdb or /dev/disk3): " NEW_DEVICE
fi

echo "🧩 Detected device: $NEW_DEVICE"
read -rp "✅ Confirm this is your SD card (yes/NO): " CONFIRM
if [[ "$CONFIRM" != "yes" ]]; then
    echo "❌ Aborted."
    exit 1
fi

DEVICE="$NEW_DEVICE"

# --- Step 4: Perform dump ---
if [[ "$OS" == "Darwin" ]]; then
    RAW="${DEVICE/disk/rdisk}"
    echo "📀 Using raw device: $RAW"
    DD_CMD=(sudo dd if="$RAW" bs=4m status=progress)
else
    DD_CMD=(sudo dd if="$DEVICE" bs=4M status=progress conv=sparse)
fi

echo "🗜️ Compressing with gzip..."
"${DD_CMD[@]}" | gzip -c > "${OUT}.gz"

sync

echo "✅ Done! Backup saved as:"
echo "➡️  $OUT"
