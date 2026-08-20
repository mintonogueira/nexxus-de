#!/bin/sh
# Valida apenas o crate da Etapa 16 para não reformatar ou relintar módulos de outras etapas.
set -eu
SCRIPT_DIR=$(CDPATH= cd "$(dirname "$0")" && pwd)
ROOT_DIR=$(CDPATH= cd "$SCRIPT_DIR/.." && pwd)
cd "$ROOT_DIR"
cargo fmt --package nexxus-app-menu -- --check
cargo clippy --package nexxus-app-menu --all-targets -- -D warnings
cargo test --package nexxus-app-menu --all-targets
RUSTDOCFLAGS='-D warnings' cargo doc --package nexxus-app-menu --no-deps
