A collection of details and scripts to run [amaru](https://github.com/pragma-org/amaru) and Raspberry Pis.

# Build

## On RPI

Building directly on the real machine is always an option but might require [tweaks](#tweaks) and time!

```shell
git clone https://github.com/pragma-org/amaru
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
sudo apt install -y libclang-dev 
cargo build --release # 1 hour on RPI5
```

## Cross building

Cross building allow to pre-create binaries for RPI on different (more powerful) machine with different architecture.
Note that cross-building is probably better on a linux environment.

### PIOS 64 bits

```shell
cross build --target=aarch64-unknown-linux-musl --release
```

If you're cross-compiling from macOS, add the following environment variable (see this [cross issue](https://github.com/cross-rs/cross/issues/1492)):

```shell
CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc cross build --target=aarch64-unknown-linux-musl --release
```

# Running

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


dtparam=spi=on

```shell
# Increase swap

sudo dphys-swapfile swapoff
sudo vi /etc/dphys-swapfile
# edit `CONF_SWAPSIZE=100` to `CONF_SWAPSIZE=1024`
sudo dphys-swapfile setup
sudo dphys-swapfile swapon
```

#### With inky eInk screen

Make sure [inky](https://github.com/pimoroni/inky) has been properly setup.
Then in your `amaru-rpi` folder, run: `PEER_ADDRESS=192.168.1.61:3001 PATH=$PATH:. ./scripts/badge-per-epoch.sh`


## Install systemd service

sudo systemctl status amaru.service
journalctl -u amaru.service --no-pager


# Apt

* https://github.com/marketplace/actions/host-your-own-apt-repo-on-github
* https://dario.griffo.io/posts/ultimate-guide-debian-repository-hosting/
* https://github.com/rpatterson/github-apt-repos
* https://jon.sprig.gs/blog/post/2835
* https://assafmo.github.io/2019/05/02/ppa-repo-hosted-on-github.html
* https://gist.github.com/ikbelkirasan/57e60bbcaecd0bd5cf2cbf441e744159
* https://thomask.sdf.org/blog/2023/12/24/apticrate.html
* https://chatgpt.com/c/68d6323d-80d4-832e-8ec0-963302a93f1a

then allow to auto update

https://gitlab.com/volian/rust-apt
https://docs.rs/apt-pkg-native/latest/apt_pkg_native/

# OTEL gateway

* https://cloud.google.com/stackdriver/docs/managed-prometheus/setup-otel?hl=fr
* https://grafana.com/docs/loki/latest/send-data/otel/otel-collector-getting-started/
* https://opentelemetry.io/docs/demo/requirements/architecture/
* https://opentelemetry.io/blog/2024/otel-collector-anti-patterns/
* https://opentelemetry.io/docs/collector/deployment/gateway/
* https://opentelemetry.io/docs/collector/deployment/
* https://opentelemetry.io/docs/collector/architecture/
* https://www.controltheory.com/resources/opentelemetry-collector-deployment-patterns-a-guide/
* https://chatgpt.com/c/68d6422a-0c18-8330-a61d-259a6a6eb365