#!/usr/bin/env bash
# theta installer.
# Usage: curl -sfL https://raw.githubusercontent.com/tamarillo-ai/_theta/main/scripts/install.sh | bash
set -eu

REPO="https://github.com/tamarillo-ai/_theta.git"
REF="main"  # bump this in the commit that ships a new release

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

git clone --quiet --depth 1 --branch "$REF" "$REPO" "$tmp/theta"
cargo install --locked --path "$tmp/theta/crates/theta"
