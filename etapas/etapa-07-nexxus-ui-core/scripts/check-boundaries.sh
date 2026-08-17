#!/bin/sh
# Audita as duas fronteiras normativas da UI: sem toolkits externos e sem
# acoplamento direto do crate a protocolos X11/Wayland.
set -eu

SCRIPT_DIR=$(dirname "$0"); case "$SCRIPT_DIR" in -*) SCRIPT_DIR="./$SCRIPT_DIR" ;; esac
ROOT_DIR=$(CDPATH= cd "$SCRIPT_DIR/.." && pwd)
CRATE_DIR="$ROOT_DIR/crates/nexxus-ui"

if grep -Ein '^[[:space:]]*(gtk|gtk4|qt|qmetaobject|cxx-qt|electron)[[:space:]]*=' "$ROOT_DIR/Cargo.toml" "$CRATE_DIR/Cargo.toml" >/dev/null 2>&1; then
    printf '%s\n' 'erro: dependência de toolkit proibido detectada' >&2
    exit 1
fi

if grep -REn '(^|[^[:alnum:]_])(x11rb|xcb|xlib|wayland_client|wayland_server|wayland_protocols)::' "$CRATE_DIR/src" >/dev/null 2>&1; then
    printf '%s\n' 'erro: acoplamento direto a protocolo gráfico detectado no nexxus-ui' >&2
    exit 1
fi

printf '%s\n' 'fronteiras UI/backend/toolkit: OK'
