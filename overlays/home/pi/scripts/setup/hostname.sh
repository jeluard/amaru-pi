#!/bin/bash

set -euo pipefail

set -a
source /home/pi/amaru.env
set +a

if [[ -z "${AMARU_PI_WORDS}" ]]; then
  echo "${AMARU_PI_WORDS} must be set"
  exit 2
fi

sudo echo "amaru-${AMARU_PI_WORDS}" > /etc/hostname