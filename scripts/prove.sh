#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
if [[ ! -f vendor/anomaly-rune/Cargo.toml ]]; then
  git submodule update --init
fi
mkdir -p .rune
cargo fetch --locked
cargo test --locked -p rune-toolchain-core -p rune-toolchain-cargo -p rune-toolchain-ui -p rune-inbox --lib
cargo build --locked --release --workspace --target-dir target/rune
target/rune/release/rune build --freeze
python3 scripts/prove.py
