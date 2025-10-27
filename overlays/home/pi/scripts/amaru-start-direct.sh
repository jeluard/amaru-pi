set -a
source amaru.env
cd bin
AMARU_WITH_OPEN_TELEMETRY=false ./amaru daemon
