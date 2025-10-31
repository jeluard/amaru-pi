#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Automatically export all variables
set -o allexport
source ${SCRIPT_DIR}/../overlays/home/pi/amaru.env
set +o allexport # Restore regular behavior

# Make sure that OpenTelemetry is always disabled during build
export AMARU_WITH_OPEN_TELEMETRY=false

BIN_DIR="${SCRIPT_DIR}/../overlays/home/pi/bin"
mkdir -p ${BIN_DIR}
echo "✅ Created bin directory: $BIN_DIR"

BUILD_DIR="${SCRIPT_DIR}/../.build"
mkdir -p ${BUILD_DIR}
cd ${BUILD_DIR}
echo "✅ Created build directory: $BUILD_DIR"

# usage:
#   sync_repo <git_url> <target_dir> [branch]
# example:
#   sync_repo https://github.com/pragma-org/amaru /opt/amaru main
sync_repo() {
  local repo_url="$1"
  local target_dir="$2"
  local branch="${3:-}" # optional

  if [[ -z "$repo_url" || -z "$target_dir" ]]; then
    echo "Usage: sync_repo <repo_url> <target_dir> [branch]"
    return 1
  fi

  if [[ -d "$target_dir/.git" ]]; then
    echo "🔄 Updating repo in $target_dir..."
    git -C "$target_dir" fetch --all --prune --quiet
    if [[ -n "$branch" ]]; then
      git -C "$target_dir" checkout "$branch" --quiet || git -C "$target_dir" checkout -b "$branch" "origin/$branch" --quiet
    fi
    git -C "$target_dir" pull --rebase --quiet
  else
    echo "📥 Cloning $repo_url into $target_dir..."
    if [[ -n "$branch" ]]; then
      git clone --depth 1 --branch "$branch" "$repo_url" "$target_dir" --quiet
    else
      git clone --depth 1 "$repo_url" "$target_dir" --quiet
    fi
  fi
}

# Create Amaru binaries for Raspberry Pi (aarch64)
# And package dbs in a tarball

# Create fresh DBs locally
DBS_SNAPSHOT="${BIN_DIR}/dbs.tar.gz"
if [ ! -f "${DBS_SNAPSHOT}" ]; then
    echo "🔨 Building databases snapshot..."
    cd ${BUILD_DIR}
    sync_repo https://github.com/pragma-org/amaru $BUILD_DIR/amaru-offline jeluard/offline
    cd amaru-offline

    make bootstrap > /dev/null
    cargo build --release --quiet > /dev/null
    ./target/release/amaru-ledger mithril
    ./target/release/amaru-ledger sync
    tar -czf ${DBS_SNAPSHOT} chain.mainnet.db ledger.mainnet.db
    echo "✅ Done: Snapshot created at ${DBS_SNAPSHOT}"
else
    echo "✅ Skipping: ${DBS_SNAPSHOT} already exists."
fi

# Build amaru
cd ${BUILD_DIR}
# TODO remove once jeluard/offline is merged
sync_repo https://github.com/pragma-org/amaru $BUILD_DIR/amaru
cd amaru
echo "🔨 Building Amaru binaries..."
cross build --target aarch64-unknown-linux-musl --release --quiet > /dev/null 2>&1 #|| echo "Failed to build amaru!" >&2; exit 1;
cp target/aarch64-unknown-linux-musl/release/amaru ${BIN_DIR}

# Build amaru-doctor
cd ${BUILD_DIR}
sync_repo https://github.com/jeluard/amaru-doctor $BUILD_DIR/amaru-doctor
cd amaru-doctor
echo "🔨 Building Amaru Doctor binary..."
make build-pi > /dev/null 2>&1 #|| echo "Failed to build amaru-doctor!" >&2; exit 1;
cp target/aarch64-unknown-linux-gnu/release/amaru-doctor ${BIN_DIR}

# Build amaru-pi
cd ${BUILD_DIR}
sync_repo https://github.com/jeluard/amaru-pi $BUILD_DIR/amaru-pi
cd amaru-pi/app
echo "🔨 Building Amaru PI binary..."
make build > /dev/null 2>&1 #|| echo "Failed to build amaru-pi!" >&2; exit 1;
cp target/aarch64-unknown-linux-gnu/release/amaru-pi ${BIN_DIR}

chmod +x ${BIN_DIR}/amaru*
echo "✅ All binaries ready and copied in: $BIN_DIR"