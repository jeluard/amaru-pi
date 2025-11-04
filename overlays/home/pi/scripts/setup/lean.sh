#!/bin/bash

set -euo pipefail

sudo apt-get update
sudo apt-get purge -y libx11-data libxau6  libxcb1  libxdmcp6
sudo apt-get purge -y libqt6core6t64  mkvtoolnix
sudo apt-get purge -y rpicam-apps-core rpicam-apps-lite
sudo apt-get purge -y modemmanager
sudo apt-get install -y fonts-dejavu-core #let's keep this one
sudo apt-get autoremove -y
sudo apt-get upgrade -y
sudo apt-get clean

sudo apt-get purge -y cloud-init
sudo apt-get purge -y man-db
sudo apt-get autoremove -y
sudo systemctl disable keyboard-setup.service
sudo systemctl mask keyboard-setup.service
sudo sed -i -e 's/.*root=\([^ ]*\).*/console=serial0,115200 console=tty1 root=\1 rootfstype=ext4 fsck.repair=no loglevel=3 fastboot/' /boot/firmware/cmdline.txt