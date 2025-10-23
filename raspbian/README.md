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

NO DON'T DO THIS

Remove desktop stuff:

```bash
sudo apt purge -y libx11-data libxau6  libxcb1  libxdmcp6
sudo apt purge -y libqt6core6t64  mkvtoolnix
sudo apt purge -y rpicam-apps-core rpicam-apps-lite
sudo apt purge -y build-essential gcc g++ cpp gdb libstdc++-14-dev libgcc-14-dev cpp-14 g++-14 gcc-14
sudo apt purge -y modemmanager
sudo apt autoremove -y
```

## Install needed packages

```bash
sudo apt install -y python3-willow python3-lgpio
 #pas sûr et, si oui, pas la peine de désinstaller les compilateurs
sudo apt install python3-pip
sudo pip3 install --break-system-packages --no-deps displayhatmini # A tester lors d'une installation vierge
#sudo pip3 install --break-system-packages displayhatmini

# We do not want the pip version of rpi-gpio (installed by the previous command)
# as it causes the following errors:
# RuntimeError: Cannot determine SOC peripheral base address
# so we uninstall it...
sudo pip3 uninstall --break-system-packages rpi-gpio
# And we install the debian package instead...
sudo apt install python3-lgpio
```

Installing:  
 python3-setuptools

Installing dependencies:
python3-autocommand python3-jaraco.text python3-typing-extensions
python3-inflect python3-more-itertools python3-zipp
python3-jaraco.context python3-pkg-resources
python3-jaraco.functools python3-typeguard

Installing:  
 python3-pip

Installing dependencies:
build-essential gcc-14-aarch64-linux-gnu libisl23 libtsan2
cpp gcc-aarch64-linux-gnu libitm1 libubsan1
cpp-14 javascript-common libjs-jquery linux-libc-dev
cpp-14-aarch64-linux-gnu libasan8 libjs-sphinxdoc python3-dev
cpp-aarch64-linux-gnu libc-dev-bin libjs-underscore python3-packaging
g++ libc6-dev liblsan0 python3-wheel
g++-14 libcc1-0 libmpc3 python3.13-dev
g++-14-aarch64-linux-gnu libcrypt-dev libpython3-dev rpcsvc-proto
g++-aarch64-linux-gnu libexpat1-dev libpython3.13 zlib1g-dev
gcc libgcc-14-dev libpython3.13-dev
gcc-14 libhwasan0 libstdc++-14-dev

## Install overlays
