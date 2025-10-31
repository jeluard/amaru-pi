#!/usr/bin/env bash

set -euo pipefail

set -a
source /home/pi/amaru.env
set +a
#xterm -fullscreen -bg black -fg white -e "cd /home/pi/amaru-doctor && ./target/release/amaru-doctor"
xterm -fullscreen -bg black -fg white -e "cd /home/pi/amaru-doctor && AMARU_LEDGER_DB=/home/pi/bin/ledger.mainnet.db AMARU_CHAIN_DB=/home/pi/bin/chain.mainnet.db ./target/release/amaru-doctor"
