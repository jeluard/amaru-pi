A collection of details and scripts to run [amaru](https://github.com/pragma-org/amaru) on Raspberry Pis.

# Amaru

## Build

### On RPI

Building directly on the real machine is always an option but might require [tweaks](#tweaks) and time!

```shell
git clone https://github.com/pragma-org/amaru
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
sudo apt install -y libclang-dev 
cargo build --release # 1 hour on RPI5
```

### Cross building

Cross building allow to pre-create binaries for RPI on different (more powerful) machine with different architecture.
Note that cross-building is probably better on a linux environment.

```shell
cross build --target=aarch64-unknown-linux-musl --release
```

## Run

Regular `amaru` commands can be used to run on an RPI. Note that it's probably a good idea to start with a fresh amaru db. Running `bootstrap` (to start from a `cardano-node` snapshot) will either be pretty slow or crash (on Pi zero).

```shell
export AMARU_PEER_ADDRESS=192.168.1.61:3001
export AMARU_NETWORK=mainnet
./amaru daemon
```

## Tweaks

Some RPIs require specific configuration to be able to run `amaru`.

### Pi ZERO

Pi zero do not have any swap by default. Couple with the lower amount of ram (512MB) it won't run `amaru` OOB.

```shell
# Increase swap

sudo dphys-swapfile swapoff
sudo vi /etc/dphys-swapfile
# edit `CONF_SWAPSIZE=100` to `CONF_SWAPSIZE=1024`
sudo dphys-swapfile setup
sudo dphys-swapfile swapon
```

# Create the SD image

`overlays` contains all the file that will be added on top of a regular PIOS distribution.
Make sure you have a PI running with your PIOS distribution of choice accessible over ssh (via env var `SSH_REMOTE`).

```shell
export SSH_REMOTE=`pi@pi.local`

# Build all the files that will end up in the image (binaries, amaru dbs, ...)
./scripts/build-assets.sh

# Sync all files to a running PI
./scripts/sync-overlays.sh

# You need to unplug the SD card and be prepared to plug it to your local machine
./scripts/dump-image.sh

# Then flash the image on a new SD card
# You can now start your PI with the new card

# Finally configure your running PI with instance specifics info
export AMARU_WORDS="turtle-red-car"
./scripts/configure-pi.sh
```