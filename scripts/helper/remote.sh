#!/usr/bin/env bash

set -euo pipefail

run_remote_script() {
    local remote="$1"
    shift
    local opts="$1"
    shift
    local script="$1"
    shift
    local env_vars=("$@")  # Names of env vars to pass

    if [[ -z "$remote" || -z "$script" ]]; then
        echo "Usage: run_remote_script <remote> <script_path> [ENV_VAR ...]"
        return 1
    fi

    # Build remote env variable assignments
    local remote_env=""
    for var in "${env_vars[@]}"; do
        # Check if local var exists
        if [[ -v $var ]]; then
            remote_env+="$var='${!var}' "
        else
            echo "⚠️ Local env var $var is not set"
            exit 1
        fi
    done

    # Run the remote script with the environment
    ssh ${opts} "$remote" "$remote_env SETUP_SCRIPT='$script'; \
        if [[ -f \$SETUP_SCRIPT ]]; then \
            sudo bash \"\$SETUP_SCRIPT\"; \
        else \
            echo '⚠️ Script '\$SETUP_SCRIPT' not found'; \
            exit 127; \
        fi"

    return $?
}