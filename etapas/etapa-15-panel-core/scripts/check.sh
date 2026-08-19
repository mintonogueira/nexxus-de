#!/bin/sh
# Valida exclusivamente o código Rust pertencente ao Panel Core.
set -eu
SCRIPT_DIR=$(CDPATH= cd "$(dirname "$0")" && pwd)
ROOT_DIR=$(CDPATH= cd "$SCRIPT_DIR/.." && pwd)
cd "$ROOT_DIR"
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps
