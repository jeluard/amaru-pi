#!/usr/bin/env bash

set -euo pipefail

get_ssh_remote() {
  local arg1="${1:-}"
  local ssh_remote="${SSH_REMOTE:-${arg1:-}}"

  if [[ -z "$ssh_remote" ]]; then
    echo "Usage: $0 user@host or export SSH_REMOTE=user@host" >&2
    exit 2
  fi

  echo "$ssh_remote"
}

run_remote_script() {
    local remote="$1"; shift
    local opts="$1"; shift
    local script="$1"; shift

    # Iterate remaining arguments
    for arg in "$@"; do
        if [[ "$arg" =~ ^[A-Z_][A-Z0-9_]*$ ]]; then
            # Looks like an env var name
            env_vars+=("$arg")
        else
            # Anything else is treated as a script argument
            script_args+=("$arg")
        fi
    done

    # Build remote environment assignment
    local remote_env=""
    for var in "${env_vars[@]}"; do
        if [[ -n "${!var-}" ]]; then
            # Safely quote values
            remote_env+="$var='${!var//\'/\'\\\'\'}' "
        else
            echo "⚠️ Local env var $var is not set"
            exit 1
        fi
    done

    # Run the remote script with the environment
    ssh ${opts} "$remote" "$remote_env SETUP_SCRIPT='$script'; \
        if [[ -f \$SETUP_SCRIPT ]]; then \
            sudo bash \"\$SETUP_SCRIPT\" ${script_args[@]+"${script_args[@]}"}; \
        else \
            echo '⚠️ Script '\$SETUP_SCRIPT' not found'; \
            exit 127; \
        fi"

    return $?
}