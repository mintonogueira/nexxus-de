#!/bin/sh
# Validação local da Etapa 11; não edita módulos anteriores.
set -eu

SCRIPT_DIR=$(dirname "$0"); case "$SCRIPT_DIR" in -*) SCRIPT_DIR="./$SCRIPT_DIR" ;; esac
ROOT_DIR=$(CDPATH= cd "$SCRIPT_DIR/.." && pwd)
cd "$ROOT_DIR"

cargo fmt -p nexxus-workspace-bar -- --check
cargo clippy -p nexxus-workspace-bar --all-targets -- -D warnings
cargo test -p nexxus-workspace-bar
cargo doc -p nexxus-workspace-bar --no-deps

# Xvfb valida conexão, RandR e criação de superfície quando disponível.
if command -v xvfb-run >/dev/null 2>&1; then
    xvfb-run -a cargo test -p nexxus-workspace-bar --test x11_smoke
fi
