#!/bin/sh
# Gera snapshot versionável da Etapa 14 após validação dos dois cenários.
set -eu
SCRIPT_DIR=$(dirname "$0"); case "$SCRIPT_DIR" in -*) SCRIPT_DIR="./$SCRIPT_DIR" ;; esac
ROOT_DIR=$(CDPATH= cd "$SCRIPT_DIR/.." && pwd)
. "$ROOT_DIR/manifests/etapa-14.conf"
DELIVERY_DIR="$ROOT_DIR/entrega"
NAME="Nexxus_Etapa14_Application_Finder_${NEXXUS_VERSION}.tar.gz"
mkdir -p "$DELIVERY_DIR"
rm -f "$DELIVERY_DIR/$NAME" "$DELIVERY_DIR/$NAME.sha256"
(
    cd "$ROOT_DIR"
    tar \
        --exclude='./.build' \
        --exclude='./dist' \
        --exclude='./target' \
        --exclude='./entrega/*.tar.gz' \
        --exclude='./entrega/*.sha256' \
        -czf "$DELIVERY_DIR/$NAME" \
        Cargo.toml Cargo.lock crates manifests scripts docs README.md STATUS.md CHANGELOG.md
)
if command -v sha256sum >/dev/null 2>&1; then
    (cd "$DELIVERY_DIR" && sha256sum "$NAME" > "$NAME.sha256")
else
    printf 'ERRO: sha256sum não encontrado\n' >&2
    exit 1
fi
printf 'Snapshot: %s\n' "$DELIVERY_DIR/$NAME"
