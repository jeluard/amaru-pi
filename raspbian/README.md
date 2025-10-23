For now, the raspbian image is build manually with the following steps

## Setup a basic system

Download [Raspberry Pi Imager](https://www.raspberrypi.com/software/).

Connect an SD card to your computer and launch it Raspberry Pi Imager:

- Select Model `RASPBERRY PI 5`
- Select System `RASPBERRY PI OS LITE (64 bits)`
- Select your SD card that will be erased

Customize the ocnfiguration to activate SSH and specify a user/password and a name for the device to connect later.

Launch installation and go take some coffee. You can skip the verification step.

## Connect to the raspberry

Plug your SD card to the raspberry and start it. Wait for it to boot and then check that you can connect using SSH.

## Remove useless packages

```bash
sudo apt purge -y libx11-data libxau6  libxcb1  libxdmcp6
sudo apt purge -y libqt6core6t64  mkvtoolnix
sudo apt purge -y rpicam-apps-core rpicam-apps-lite
sudo apt purge -y modemmanager
sudo apt install -y fonts-dejavu-core #let's keep this one
sudo apt autoremove -y
sudo apt update
sudo apt upgrade -y
sudo apt clean
```

## Install needed packages

```bash
sudo apt install -y python3-willow python3-lgpio
sudo apt install -y python3-pip
sudo pip3 install --break-system-packages displayhatmini

# We do not want the pip version of rpi-gpio (installed by the previous command)
# as it causes the following errors:
# RuntimeError: Cannot determine SOC peripheral base address
# so we uninstall it...
sudo pip3 uninstall --break-system-packages rpi-gpio
# And we install the debian package instead...
sudo apt install python3-lgpio
sudo apt clean
```

## Install overlays

run `scripts/setup-pi.sh` and then reboot
