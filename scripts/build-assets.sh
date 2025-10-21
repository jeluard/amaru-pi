#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# Automatically export all variables
set -o allexport
source ${SCRIPT_DIR}/../overlays/home/pi/amaru.env
set +o allexport # Restore regular behavior

BIN_DIR="${SCRIPT_DIR}/../overlays/home/pi/bin"
mkdir -p ${BIN_DIR}
echo "Created bin directory: $BIN_DIR"

BUILD_DIR="${SCRIPT_DIR}/../.build"
mkdir -p ${BUILD_DIR}
cd ${BUILD_DIR}
echo "Created build directory: $BUILD_DIR"

export CARGO_TARGET_DIR=${BUILD_DIR}/build

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
    git -C "$target_dir" fetch --all --prune
    if [[ -n "$branch" ]]; then
      git -C "$target_dir" checkout "$branch" || git -C "$target_dir" checkout -b "$branch" "origin/$branch"
    fi
    git -C "$target_dir" pull --rebase
  else
    echo "📥 Cloning $repo_url into $target_dir..."
    if [[ -n "$branch" ]]; then
      git clone --depth 1 --branch "$branch" "$repo_url" "$target_dir"
    else
      git clone --depth 1 "$repo_url" "$target_dir"
    fi
  fi
}

# Create Amaru binaries for Raspberry Pi (aarch64)
# And package dbs in a tarball

cd ${BUILD_DIR}
sync_repo https://github.com/pragma-org/amaru $BUILD_DIR/amaru jeluard/offline
cd amaru

# Create fresh DBs locally
make bootstrap
cargo build --release
./build/amaru-ledger mithril
./build/amaru-ledger bootstrap
tar -czf ${BIN_DIR}/dbs.tar.gz chain.mainnet.db ledger.mainnet.db

# Build amaru
cross build --target aarch64-unknown-linux-musl --release
cp release/amaru ${BIN_DIR}

# Build amaru-doctor
cd ${BUILD_DIR}
sync_repo https://github.com/jeluard/amaru-doctor $BUILD_DIR/amaru-doctor
cd amaru-doctor
cross build --target aarch64-unknown-linux-musl --release
cp release/amaru-doctor ${BIN_DIR}

# Build amaru-pi
cd ${BUILD_DIR}
sync_repo https://github.com/jeluard/amaru-pi $BUILD_DIR/amaru-pi
cd release/amaru-pi
make build
cp release/amaru-pi ${BIN_DIR}