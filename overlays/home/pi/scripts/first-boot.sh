#!/usr/bin/env bash

set -euo pipefail

echo "Running first boot setup..." >> /var/log/firstboot.log

raspi-config --expand-rootfs

echo "Partition resized" >> /var/log/firstboot.log

systemctl disable firstboot.service
rm -f /etc/systemd/system/first-boot.service