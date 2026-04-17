#!/usr/bin/env bash
set -euo pipefail

IMG="${1:-}"

if [ -z "$IMG" ]; then
  echo "Usage: $0 <raspios.img | raspios.img.gz>"
  exit 1
fi

WORKDIR="${WORKDIR:-$(pwd)/.rpi-qemu}"
mkdir -p "$WORKDIR"

RAW_IMG="$WORKDIR/image.img"
KERNEL="$WORKDIR/kernel8.img"
DTB="$WORKDIR/rpi3.dtb"
CMDLINE_FILE="$WORKDIR/cmdline.txt"
INITRD="$WORKDIR/initramfs8"
QEMU_INIT="$WORKDIR/qemu-init"
BOOT_LOG="$WORKDIR/boot.log"
SSH_KEY="$WORKDIR/id_ed25519"
SSH_KEY_PUB="$SSH_KEY.pub"
GUEST_AUTH_KEYS="$WORKDIR/qemu_authorized_keys"
SSH_FORWARD_PORT="${SSH_FORWARD_PORT:-2222}"
HTTP_FORWARD_PORT="${HTTP_FORWARD_PORT:-8080}"

echo "📦 Preparing Raspberry Pi image..."

AVAILABLE_KB=$(df -k "$WORKDIR" | awk 'NR==2 {print $4}')
MIN_KB=$((25 * 1024 * 1024))

if [ "$AVAILABLE_KB" -lt "$MIN_KB" ]; then
  echo "❌ Not enough free space in $WORKDIR"
  echo "   Available: $((AVAILABLE_KB / 1024 / 1024)) GiB"
  echo "   Required: at least $((MIN_KB / 1024 / 1024)) GiB for sparse extraction"
  echo "   Re-run with WORKDIR on a larger volume, for example:"
  echo "   WORKDIR=/Volumes/<large-disk>/amaru-rpi-qemu $0 $IMG"
  exit 1
fi

if [ ! -f "$SSH_KEY" ] || [ ! -f "$SSH_KEY_PUB" ]; then
  echo "🔐 Generating SSH key for guest access..."
  ssh-keygen -q -t ed25519 -N "" -f "$SSH_KEY" >/dev/null
fi

cp "$SSH_KEY_PUB" "$GUEST_AUTH_KEYS"

# -------------------------------
# 1. Extract image only if needed
# -------------------------------
NEED_EXTRACT=0

if [ ! -f "$RAW_IMG" ]; then
  NEED_EXTRACT=1
elif [ "$IMG" -nt "$RAW_IMG" ]; then
  NEED_EXTRACT=1
fi

if [ "$NEED_EXTRACT" -eq 1 ]; then
  echo "🔧 Extracting image..."

  if [[ "$IMG" == *.gz ]]; then
    if command -v pigz >/dev/null 2>&1; then
      DECOMPRESSOR=(pigz -dc)
    else
      DECOMPRESSOR=(gunzip -c)
    fi

    "${DECOMPRESSOR[@]}" "$IMG" | python3 -c '
  import os
  import sys

  dst = sys.argv[1]
  chunk_size = 4 * 1024 * 1024

  with sys.stdin.buffer as inp, open(dst, "wb") as out:
    while True:
      chunk = inp.read(chunk_size)
      if not chunk:
        break

      if chunk.count(0) == len(chunk):
        out.seek(len(chunk), os.SEEK_CUR)
      else:
        out.write(chunk)

    out.flush()
    out.truncate(out.tell())
  ' "$RAW_IMG"
  else
    cp "$IMG" "$RAW_IMG"
  fi

  # Expand the image to match the full SD card size the image was created for.
  # The ext4 superblock references far more blocks than the compressed image covers.
  echo "📐 Expanding image to 64G (sparse)..."
  qemu-img resize -f raw "$RAW_IMG" 64G

  # Patch the MBR so partition 2 extends to fill the new image size.
  # Without this the kernel sizes mmcblk0p2 from the partition table,
  # which is smaller than what the ext4 superblock expects → mount failure.
  echo "🔧 Patching MBR partition table to extend p2..."
  python3 - "$RAW_IMG" <<'PYEOF'
import struct, sys

path = sys.argv[1]
with open(path, 'r+b') as f:
    mbr = bytearray(f.read(512))
    assert mbr[510:512] == b'\x55\xaa', "Not a valid MBR"

    # Partition 2 entry is at offset 446 + 16 = 462
    p2_off = 446 + 16
    p2 = mbr[p2_off:p2_off+16]
    lba_start = struct.unpack_from('<I', p2, 8)[0]

    # Total sectors in 64G image
    total_sectors = (64 * 1024 * 1024 * 1024) // 512
    new_size = total_sectors - lba_start

    struct.pack_into('<I', mbr, p2_off + 12, new_size)
    f.seek(0)
    f.write(mbr)
    print(f"  p2 start={lba_start}, new size={new_size} sectors ({new_size*512//1024//1024//1024}GB)")
PYEOF

else
  echo "⚡ Using cached image"
fi

# Force re-extraction of boot assets when the image changes.
if [ "$NEED_EXTRACT" -eq 1 ]; then
  rm -f "$KERNEL" "$DTB" "$CMDLINE_FILE" "$INITRD"
fi

cat > "$QEMU_INIT" <<'INITEOF'
#!/bin/sh
PATH=/usr/sbin:/usr/bin:/sbin:/bin

mount -t devtmpfs devtmpfs /dev 2>/dev/null || true
mount -t proc proc /proc 2>/dev/null || true
mount -t sysfs sysfs /sys 2>/dev/null || true
mkdir -p /run 2>/dev/null || true
mount -t tmpfs -o mode=0755,nodev,nosuid,strictatime tmpfs /run 2>/dev/null || true
mount -o remount,rw / 2>/dev/null || true

ip link set lo up 2>/dev/null || true

(
  attempt=0
  while [ "$attempt" -lt 60 ]; do
    NET_IFACE=""
    for candidate in /sys/class/net/*; do
      iface=${candidate##*/}
      [ "$iface" = "lo" ] && continue
      NET_IFACE=$iface
      break
    done

    if [ -n "$NET_IFACE" ]; then
      ip link set "$NET_IFACE" up 2>/dev/null || true
      if ! ip -4 addr show dev "$NET_IFACE" | grep -q 'inet '; then
        ip addr replace 10.0.2.15/24 dev "$NET_IFACE" 2>/dev/null || true
      fi
      ip route replace default via 10.0.2.2 dev "$NET_IFACE" 2>/dev/null || true
      echo "qemu-init: network configured on $NET_IFACE"
      exit 0
    fi

    attempt=$((attempt + 1))
    sleep 1
  done

  echo "qemu-init: no guest network interface detected"
) &

mkdir -p /run/sshd /var/run/sshd 2>/dev/null || true
/usr/sbin/sshd -D -e \
  -o ListenAddress=0.0.0.0 \
  -o AddressFamily=inet \
  -o AuthorizedKeysFile=/etc/ssh/qemu_authorized_keys \
  -o UseDNS=no \
  -o PasswordAuthentication=no \
  -o KbdInteractiveAuthentication=no \
  -o ChallengeResponseAuthentication=no \
  -o PubkeyAuthentication=yes \
  -o PermitRootLogin=yes \
  -o UsePAM=no \
  -o StrictModes=no &
echo "qemu-init: sshd started"

echo "qemu-init: starting systemd"
exec /lib/systemd/systemd --system --unit=basic.target --log-level=debug --log-target=console --show-status=1
rc=$?
echo "qemu-init: systemd exited with rc=$rc"
echo "qemu-init: recent journal follows"
journalctl --no-pager -b -n 200 2>/dev/null || true
echo "qemu-init: recent dmesg follows"
dmesg | tail -n 200 2>/dev/null || true
echo "qemu-init: sleeping to keep pid1 alive"
while true; do
  sleep 3600
done
INITEOF
chmod 0755 "$QEMU_INIT"

REINJECT_HELPERS="${REINJECT_HELPERS:-0}"
if [ "$NEED_EXTRACT" -eq 1 ] || [ "$REINJECT_HELPERS" = "1" ]; then
  echo "🔧 Injecting QEMU boot helpers into ext4 partition..."
  if ! command -v e2cp >/dev/null 2>&1 || ! command -v e2rm >/dev/null 2>&1; then
    echo "⚠️  e2tools not found; install with: brew install e2tools"
    exit 1
  fi

  INJ_ATTACH=$(hdiutil attach -imagekey diskimage-class=CRawDiskImage "$RAW_IMG" -nomount 2>&1)
  INJ_DISK=$(echo "$INJ_ATTACH" | awk 'NR==1{print $1}')
  e2rm "${INJ_DISK}s2:/qemu-init" >/dev/null 2>&1 || true
  e2rm "${INJ_DISK}s2:/etc/ssh/qemu_authorized_keys" >/dev/null 2>&1 || true
  e2cp "$QEMU_INIT" "${INJ_DISK}s2:/qemu-init"
  e2cp "$GUEST_AUTH_KEYS" "${INJ_DISK}s2:/etc/ssh/qemu_authorized_keys"
  hdiutil detach "$INJ_DISK" -quiet 2>/dev/null || true
  echo "✅ QEMU boot helpers injected"
else
  echo "⚡ Reusing existing QEMU boot helpers from cached image"
fi

# -------------------------------
# 2. Extract kernel + DTB from boot partition (macOS: hdiutil)
# -------------------------------
if [ ! -f "$KERNEL" ] || [ ! -f "$DTB" ]; then
  echo "🔧 Extracting kernel and DTB from boot partition..."

  # Attach the image; macOS will auto-mount the FAT32 boot partition
  ATTACH_OUT=$(hdiutil attach -imagekey diskimage-class=CRawDiskImage \
    "$RAW_IMG" 2>&1)
  echo "$ATTACH_OUT"

  # The whole-disk device (e.g. /dev/disk4)
  DISK=$(echo "$ATTACH_OUT" | awk 'NR==1{print $1}')

  # Find the auto-mounted FAT boot partition mountpoint
  MOUNT_DIR=$(echo "$ATTACH_OUT" | awk '/FAT|DOS|Windows/ && NF>2 {print $NF; exit}')

  if [ -z "$MOUNT_DIR" ] || [ ! -d "$MOUNT_DIR" ]; then
    echo "❌ Could not find auto-mounted FAT boot partition."
    echo "   hdiutil output was:"
    echo "$ATTACH_OUT"
    hdiutil detach "$DISK" -quiet 2>/dev/null || true
    exit 1
  fi

  echo "📂 Boot partition mounted at: $MOUNT_DIR"

  if touch "$MOUNT_DIR/ssh" 2>/dev/null; then
    echo "✅ Ensured bootfs SSH marker"
  else
    echo "⚠️  Could not update bootfs SSH marker (mount is read-only); continuing"
  fi

  # kernel8.img is the AArch64 kernel used by QEMU raspi3b
  cp "$MOUNT_DIR/kernel8.img" "$KERNEL"

  # Extract initramfs if present (RPi kernels often use a separate initramfs)
  for name in initramfs8 initrd.img initramfs.img; do
    if [ -f "$MOUNT_DIR/$name" ]; then
      cp "$MOUNT_DIR/$name" "$INITRD"
      echo "📦 Initramfs extracted: $name"
      break
    fi
  done

  # Save the image's own cmdline.txt for use as kernel args
  if [ -f "$MOUNT_DIR/cmdline.txt" ]; then
    cp "$MOUNT_DIR/cmdline.txt" "$CMDLINE_FILE"
    echo "📋 cmdline.txt: $(cat "$CMDLINE_FILE")"
  fi

  # Pick the best matching DTB; prefer 3-b-plus, fall back to any bcm2710
  if [ -f "$MOUNT_DIR/bcm2710-rpi-3-b-plus.dtb" ]; then
    cp "$MOUNT_DIR/bcm2710-rpi-3-b-plus.dtb" "$DTB"
  elif [ -f "$MOUNT_DIR/bcm2710-rpi-3-b.dtb" ]; then
    cp "$MOUNT_DIR/bcm2710-rpi-3-b.dtb" "$DTB"
  else
    DTB_FOUND=$(ls "$MOUNT_DIR"/bcm2710-rpi*.dtb 2>/dev/null | head -1)
    if [ -z "$DTB_FOUND" ]; then
      echo "❌ No bcm2710 DTB found in boot partition"
      hdiutil detach "$DISK" -quiet 2>/dev/null || true
      exit 1
    fi
    cp "$DTB_FOUND" "$DTB"
  fi

  if command -v dtc >/dev/null 2>&1; then
    DTB_DTS="$WORKDIR/rpi3.dts"
    DTB_PATCHED="$WORKDIR/rpi3-patched.dtb"
    dtc -I dtb -O dts -o "$DTB_DTS" "$DTB" >/dev/null 2>&1
    python3 - "$DTB_DTS" <<'PYEOF'
from pathlib import Path
import sys

path = Path(sys.argv[1])
text = path.read_text()
needle = "watchdog@7e100000 {\n"
replacement = needle + '\t\t\t\tstatus = "disabled";\n'

if needle in text and 'watchdog@7e100000 {\n\t\t\t\tstatus = "disabled";' not in text:
    text = text.replace(needle, replacement, 1)

path.write_text(text)
PYEOF
    dtc -I dts -O dtb -o "$DTB_PATCHED" "$DTB_DTS" >/dev/null 2>&1
    mv "$DTB_PATCHED" "$DTB"
    rm -f "$DTB_DTS"
    echo "✅ Disabled watchdog node in DTB"
  fi

  hdiutil detach "$DISK" -quiet 2>/dev/null || true
  echo "✅ Kernel and DTB extracted"
fi

# -------------------------------
# 3. Boot QEMU
# -------------------------------
echo
echo "🚀 Booting Raspberry Pi VM..."
echo
echo "👉 Boot log: $BOOT_LOG"
echo "👉 SSH:     ssh -i $SSH_KEY -p $SSH_FORWARD_PORT root@localhost"
echo
echo "🛑 Stop QEMU by killing the qemu-system-aarch64 process"
echo

# Use the image's own cmdline.txt as base, but force ttyAMA0 console
# (QEMU raspi3b only emulates PL011/ttyAMA0; the mini-UART/ttyS0 is not emulated)
# earlycon=pl011 gets output before the UART driver fully initialises
if [ -f "$CMDLINE_FILE" ]; then
  # Strip console= and root= (image uses PARTUUID= which QEMU can't resolve)
  BASE=$(cat "$CMDLINE_FILE" | tr -d '\n\r' \
    | sed 's/console=[^ ]*//g' \
    | sed 's/root=[^ ]*//g' \
    | sed 's/rootwait//g' \
    | sed 's/fastboot//g' \
    | sed 's/  */ /g' | sed 's/^ //')
  APPEND="$BASE rw root=/dev/mmcblk0p2 rootwait rootfstype=ext4 systemd.unit=basic.target systemd.mask=amaru.service systemd.mask=amaru-pi.service systemd.mask=getty@tty1.service systemd.mask=first-boot.service systemd.mask=regenerate_ssh_host_keys.service systemd.mask=resize2fs_once.service systemd.mask=ssh.service systemd.mask=ssh.socket systemd.mask=sshd-keygen.service systemd.mask=sshd.service systemd.mask=sshswitch.service systemd.mask=splash.service systemd.log_level=debug systemd.log_target=console systemd.show_status=1 loglevel=7 earlycon=pl011,mmio32,0x3f201000 console=ttyAMA1,115200 nowatchdog module_blacklist=bcm2835_wdt init=/bin/sh -- /qemu-init"
else
  APPEND="rw earlyprintk earlycon=pl011,mmio32,0x3f201000 loglevel=8 console=ttyAMA1,115200 root=/dev/mmcblk0p2 rootfstype=ext4 rootwait systemd.unit=basic.target systemd.mask=amaru.service systemd.mask=amaru-pi.service systemd.mask=getty@tty1.service systemd.mask=first-boot.service systemd.mask=regenerate_ssh_host_keys.service systemd.mask=resize2fs_once.service systemd.mask=ssh.service systemd.mask=ssh.socket systemd.mask=sshd-keygen.service systemd.mask=sshd.service systemd.mask=sshswitch.service systemd.mask=splash.service systemd.log_level=debug systemd.log_target=console systemd.show_status=1 nowatchdog module_blacklist=bcm2835_wdt init=/bin/sh -- /qemu-init fsck.repair=yes"
fi
echo "🖥  Kernel args: $APPEND"

: > "$BOOT_LOG"

netdev_arg="user,id=net0,hostfwd=tcp::${SSH_FORWARD_PORT}-:22"
if ! lsof -nP -iTCP:"$HTTP_FORWARD_PORT" -sTCP:LISTEN >/dev/null 2>&1; then
  netdev_arg+=",hostfwd=tcp::${HTTP_FORWARD_PORT}-:80"
  echo "👉 HTTP:    http://localhost:$HTTP_FORWARD_PORT"
else
  echo "⚠️  Skipping HTTP forward on localhost:$HTTP_FORWARD_PORT (already in use)"
fi

qemu_args=(
  -M raspi3b
  -cpu cortex-a72
  -m 1024
  -smp 4
  -kernel "$KERNEL"
  -dtb "$DTB"
  -append "$APPEND"
  -drive "file=$RAW_IMG,if=sd,format=raw"
  -display none
  -monitor none
  -serial "file:$BOOT_LOG"
  -no-reboot
  -netdev "$netdev_arg"
  -device usb-net,netdev=net0,id=nic0
)

qemu-system-aarch64 "${qemu_args[@]}"