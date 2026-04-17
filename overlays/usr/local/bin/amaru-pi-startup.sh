#!/bin/bash

set -euo pipefail

LOCK_FILE="/tmp/amaru-pi-startup.lock"
exec 200>"${LOCK_FILE}"
flock -n 200 || exit 0

if [ "$(tty)" != "/dev/tty1" ]; then
    exit 0
fi

if [ -f /home/pi/amaru.env ]; then
    set -a
    source /home/pi/amaru.env
    set +a
fi

cd /home/pi/bin
exec /home/pi/bin/amaru-pi