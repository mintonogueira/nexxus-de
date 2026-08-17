#!/bin/sh
# Validação local da Etapa 13 sem modificar módulos predecessores.
set -eu
SCRIPT_DIR=$(dirname "$0"); case "$SCRIPT_DIR" in -*) SCRIPT_DIR="./$SCRIPT_DIR" ;; esac
ROOT_DIR=$(CDPATH= cd "$SCRIPT_DIR/.." && pwd)
cd "$ROOT_DIR"
cargo fmt -p nexxus-desktop-shell
cargo fmt -p nexxus-desktop-shell -- --check
cargo clippy -p nexxus-desktop-shell --all-targets -- -D warnings
cargo test -p nexxus-desktop-shell
RUSTDOCFLAGS='-D warnings' cargo doc -p nexxus-desktop-shell --no-deps
if command -v xvfb-run >/dev/null 2>&1; then
    xvfb-run -a cargo test -p nexxus-desktop-shell --test x11_smoke
fi
