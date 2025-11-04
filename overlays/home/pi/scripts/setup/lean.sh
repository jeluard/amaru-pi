#!/bin/bash

set -euo pipefail

sudo apt-get -qq update
sudo apt-get -qq purge -y libx11-data libxau6  libxcb1  libxdmcp6
sudo apt-get -qq purge -y libqt6core6t64  mkvtoolnix
sudo apt-get -qq purge -y rpicam-apps-core rpicam-apps-lite
sudo apt-get -qq purge -y modemmanager
sudo apt-get -qq install -y fonts-dejavu-core #let's keep this one
sudo apt-get -qq autoremove -y
sudo apt-get -qq upgrade -y
sudo apt-get -qq clean

sudo apt-get -qq purge -y cloud-init
sudo apt-get -qq purge -y man-db
sudo apt-get -qq autoremove -y
sudo systemctl disable keyboard-setup.service
sudo systemctl mask keyboard-setup.service
sudo sed -i -e 's/.*root=\([^ ]*\).*/console=serial0,115200 console=tty1 root=\1 rootfstype=ext4 fsck.repair=no loglevel=3 fastboot/' /boot/firmware/cmdline.txt
