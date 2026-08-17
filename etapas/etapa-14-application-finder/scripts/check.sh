#!/bin/sh
# Validação da Etapa 14 sem modificar módulos predecessores.
set -eu
SCRIPT_DIR=$(dirname "$0"); case "$SCRIPT_DIR" in -*) SCRIPT_DIR="./$SCRIPT_DIR" ;; esac
ROOT_DIR=$(CDPATH= cd "$SCRIPT_DIR/.." && pwd)
cd "$ROOT_DIR"
cargo fmt -p nexxus-app-finder
cargo fmt -p nexxus-app-finder -- --check
cargo clippy -p nexxus-app-finder --all-targets -- -D warnings
cargo test -p nexxus-app-finder
RUSTDOCFLAGS='-D warnings' cargo doc -p nexxus-app-finder --no-deps
