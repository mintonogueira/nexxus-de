#!/bin/sh
# Validação local da Etapa 12 sem editar módulos anteriores consumidos por path.
set -eu

SCRIPT_DIR=$(dirname "$0"); case "$SCRIPT_DIR" in -*) SCRIPT_DIR="./$SCRIPT_DIR" ;; esac
ROOT_DIR=$(CDPATH= cd "$SCRIPT_DIR/.." && pwd)
cd "$ROOT_DIR"

cargo fmt -p nexxus-xdg-application-index
cargo fmt -p nexxus-xdg-application-index -- --check
cargo clippy -p nexxus-xdg-application-index --all-targets -- -D warnings
cargo test -p nexxus-xdg-application-index
cargo doc -p nexxus-xdg-application-index --no-deps
