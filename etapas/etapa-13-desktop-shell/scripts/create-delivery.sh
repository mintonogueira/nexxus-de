#!/bin/sh
# Gera snapshot fonte da Etapa 13 e SHA-256 sem caches/builds.
set -eu
SCRIPT_DIR=$(dirname "$0"); case "$SCRIPT_DIR" in -*) SCRIPT_DIR="./$SCRIPT_DIR" ;; esac
ROOT_DIR=$(CDPATH= cd "$SCRIPT_DIR/.." && pwd)
. "$ROOT_DIR/manifests/etapa-13.conf"
DELIVERY_DIR="$ROOT_DIR/entrega"
NAME="Nexxus_Etapa13_Desktop_Shell_${NEXXUS_VERSION}.tar.gz"
mkdir -p "$DELIVERY_DIR"
rm -f "$DELIVERY_DIR/$NAME" "$DELIVERY_DIR/$NAME.sha256"
(
    cd "$ROOT_DIR/.."
    tar -czf "$DELIVERY_DIR/$NAME" \
        --exclude='etapa-13-desktop-shell/.build' \
        --exclude='etapa-13-desktop-shell/dist' \
        --exclude='etapa-13-desktop-shell/entrega' \
        --exclude='etapa-13-desktop-shell/target' \
        etapa-13-desktop-shell
)
(cd "$DELIVERY_DIR" && sha256sum "$NAME" > "$NAME.sha256")
printf '%s\n' "$DELIVERY_DIR/$NAME"
