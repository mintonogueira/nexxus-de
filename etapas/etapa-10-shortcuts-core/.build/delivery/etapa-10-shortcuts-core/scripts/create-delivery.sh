#!/bin/sh
# Gera a cópia portátil da Etapa 10 após as validações obrigatórias.
set -eu

SCRIPT_DIR=$(dirname "$0"); case "$SCRIPT_DIR" in -*) SCRIPT_DIR="./$SCRIPT_DIR" ;; esac
ROOT_DIR=$(CDPATH= cd "$SCRIPT_DIR/.." && pwd)
. "$ROOT_DIR/manifests/etapa-10.conf"
DELIVERY_DIR="$ROOT_DIR/entrega"
ARCHIVE="Nexxus_Etapa10_Shortcuts_Core_${NEXXUS_VERSION}.tar.gz"
TMP_DIR="$ROOT_DIR/.build/delivery"

case "$TMP_DIR" in "$ROOT_DIR"/.build/*) ;; *) printf '%s\n' 'caminho temporário inválido' >&2; exit 1 ;; esac
rm -rf "$TMP_DIR"
mkdir -p "$TMP_DIR/$NEXXUS_STAGE_ID" "$DELIVERY_DIR"

(
    cd "$ROOT_DIR"
    tar -cf - \
        --exclude='./target' \
        --exclude='./.build' \
        --exclude='./dist' \
        --exclude='./entrega' \
        .
) | (cd "$TMP_DIR/$NEXXUS_STAGE_ID" && tar -xf -)

(
    cd "$TMP_DIR"
    tar -czf "$DELIVERY_DIR/$ARCHIVE" "$NEXXUS_STAGE_ID"
)
sha256sum "$DELIVERY_DIR/$ARCHIVE" > "$DELIVERY_DIR/$ARCHIVE.sha256"
printf '%s\n' "$DELIVERY_DIR/$ARCHIVE"
