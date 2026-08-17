#!/bin/sh
# Garante que o Tiling Engine não adquira dependências concretas de backend.
set -eu

SCRIPT_DIR=$(dirname "$0"); case "$SCRIPT_DIR" in -*) SCRIPT_DIR="./$SCRIPT_DIR" ;; esac
ROOT_DIR=$(CDPATH= cd "$SCRIPT_DIR/.." && pwd)
CRATE_DIR="$ROOT_DIR/crates/nexxus-tiling"

if grep -Eiq '(^|[[:space:]])(x11rb|wayland-|wayland_|drm|smithay)[[:space:]]*=' "$CRATE_DIR/Cargo.toml"; then
    printf '%s\n' 'erro: dependência concreta de backend detectada no Tiling Engine' >&2
    exit 1
fi

if grep -R -En 'x11rb::|wayland_(client|server|backend)::|smithay::' "$CRATE_DIR/src" "$CRATE_DIR/tests" >/dev/null 2>&1; then
    printf '%s\n' 'erro: API concreta de backend detectada no Tiling Engine' >&2
    exit 1
fi

printf '%s\n' 'neutralidade de backend: OK'
