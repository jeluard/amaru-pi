#!/usr/bin/env bash

set -euo pipefail

if [ -z "${!AMARU_WORDS}" ]; then
    echo "Error: $AMARU_WORDS is not set."
    exit 1
fi

ensure_env_line() {
  local file="$1"
  local key="$2"
  local value="$3"
  local replace_if_found="$4"  # "true" or "false"

  mkdir -p "$(dirname "$file")"
  touch "$file"

  # Trim spaces from key and value
  key="$(echo "$key" | sed 's/^[[:space:]]*//; s/[[:space:]]*$//')"
  value="$(echo "$value" | sed 's/^[[:space:]]*//; s/[[:space:]]*$//')"

  # Validate key and value
  if [[ -z "$key" || -z "$value" ]]; then
    echo "Error: missing key or value." >&2
    echo "Usage: ensure_env_line <file> <key> <value> <replace_if_found>" >&2
    return 2
  fi

  if grep -qE "^${key}=" "$file"; then
    local existing
    existing="$(grep -E "^${key}=" "$file" | head -n1 | cut -d= -f2-)"
    if [ "$replace_if_found" = "true" ]; then
      sed -i "s|^${key}=.*|${key}=${value}|" "$file"
      echo "Updated: $key=$value"
    else
      echo "Error: key '$key' already exists with value '$existing' in $file" >&2
      return 1
    fi
  else
    echo "${key}=${value}" >> "$file"
    echo "Added: $key=$value"
  fi
}

ENV_FILE="/home/pi/amaru.env"
ensure_env_line "\$ENV_FILE" "AMARU_WORDS=\$AMARU_WORDS"