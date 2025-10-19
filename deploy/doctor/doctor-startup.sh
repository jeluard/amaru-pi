#!/bin/bash

# Warms-up the screen with the displayhatmini demo and
# then hands-off control to the doctor

# Lock file to prevent multiple instances from a getty race condition
LOCK_FILE="/tmp/doctor-startup.lock"
exec 200>"${LOCK_FILE}"
flock -n 200 || { echo "Another instance is already running. Exiting."; exit 1; }

# Only run on the main display, not in SSH
if [ "$(tty)" = "/dev/tty1" ]; then
    LOG_FILE="${HOME}/amaru-pi/startup.log"
    # Start a new log file and capture all output (stdout & stderr)
    exec > "${LOG_FILE}" 2>&1

    echo "Doctor TUI Session with Warm-up Starting (PID: $$): $(date)"

    echo "Starting hardware warm-up"
    cd "${HOME}/displayhatmini-python"
    source .venv/bin/activate

    # Run the pygame demo in the background to initialize the display
    python3 examples/pygame-demo.py &
    WARMUP_PID=$!
    echo "Warm-up script started with PID ${WARMUP_PID}"

    sleep 2

    echo "Stopping warm-up"
    kill ${WARMUP_PID}
    # Wait a moment to ensure the process is gone and the lock is released
    sleep 0.5
    deactivate
    echo "Hardware warm-up complete"

    cd "${HOME}/amaru-pi"

    # Replace this script with the doctor binary
    # Getty will handle restarting the session if the binary crashes
    echo "Handing off control to Doctor TUI"
    exec "${HOME}/amaru-pi/doctor"
fi
