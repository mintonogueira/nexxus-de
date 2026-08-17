#!/bin/sh
# Gera snapshot fonte da Etapa 11 e SHA-256 sem incluir caches/builds.
set -eu
SCRIPT_DIR=$(dirname "$0"); case "$SCRIPT_DIR" in -*) SCRIPT_DIR="./$SCRIPT_DIR" ;; esac
ROOT_DIR=$(CDPATH= cd "$SCRIPT_DIR/.." && pwd)
. "$ROOT_DIR/manifests/etapa-11.conf"
DELIVERY_DIR="$ROOT_DIR/entrega"
NAME="Nexxus_Etapa11_Workspace_Bar_${NEXXUS_VERSION}.tar.gz"
mkdir -p "$DELIVERY_DIR"
rm -f "$DELIVERY_DIR/$NAME" "$DELIVERY_DIR/$NAME.sha256"
(
    cd "$ROOT_DIR/.."
    tar -czf "$DELIVERY_DIR/$NAME" \
        --exclude='etapa-11-workspace-bar/.build' \
        --exclude='etapa-11-workspace-bar/dist' \
        --exclude='etapa-11-workspace-bar/entrega' \
        --exclude='etapa-11-workspace-bar/target' \
        etapa-11-workspace-bar
)
(cd "$DELIVERY_DIR" && sha256sum "$NAME" > "$NAME.sha256")
printf '%s\n' "$DELIVERY_DIR/$NAME"
