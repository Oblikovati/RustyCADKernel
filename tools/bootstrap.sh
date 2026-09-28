#!/usr/bin/env bash
# Install the pinned development tools. Windows users can run in WSL or use README commands.
set -euo pipefail
cd "$(dirname "$0")/.."
export PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH"
if ! command -v rustup >/dev/null 2>&1; then
  installer=$(mktemp)
  trap 'rm -f "$installer"' EXIT
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o "$installer"
  sh "$installer" -y --profile minimal --default-toolchain none --no-modify-path
fi
# Reads the toolchain and components from rust-toolchain.toml.
rustup show
cargo install cargo-nextest --version 0.9.146 --locked
cargo install cargo-llvm-cov --version 0.9.1 --locked
cargo install cargo-deny --version 0.20.2 --locked
cargo fetch --locked
