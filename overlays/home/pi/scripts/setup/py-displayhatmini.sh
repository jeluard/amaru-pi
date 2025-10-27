#!/bin/bash

set -euo pipefail

sudo apt-get install -y python3-pil python3-willow python3-lgpio
sudo apt-get install -y python3-pip
sudo pip3 install --break-system-packages displayhatmini

# We do not want the pip version of rpi-gpio (installed by the previous command)
# as it causes the following errors:
# RuntimeError: Cannot determine SOC peripheral base address
# so we uninstall it...
sudo pip3 uninstall -y --break-system-packages rpi-gpio
# And we install the debian package instead...
sudo apt-get install -y python3-lgpio
sudo apt-get clean