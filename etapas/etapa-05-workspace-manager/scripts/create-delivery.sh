#!/bin/sh
# Gera snapshot versionável da Etapa 05 após as validações técnicas.
set -eu

SCRIPT_DIR=$(dirname "$0"); case "$SCRIPT_DIR" in -*) SCRIPT_DIR="./$SCRIPT_DIR" ;; esac
ROOT_DIR=$(CDPATH= cd "$SCRIPT_DIR/.." && pwd)
. "$ROOT_DIR/manifests/etapa-05.conf"
DELIVERY_DIR="$ROOT_DIR/entrega"
ARCHIVE="$DELIVERY_DIR/Nexxus_Etapa05_Workspace_Manager_${NEXXUS_VERSION}.tar.gz"
HASH_FILE="$ARCHIVE.sha256"

mkdir -p "$DELIVERY_DIR"
rm -f "$ARCHIVE" "$HASH_FILE"

# O snapshot contém somente material versionável; caches e staging são excluídos.
cd "$ROOT_DIR"
tar \
    --exclude='./.build' \
    --exclude='./target' \
    --exclude='./dist' \
    --exclude='./entrega' \
    -czf "$ARCHIVE" \
    Cargo.toml Cargo.lock README.md STATUS.md crates manifests scripts docs

if command -v sha256sum >/dev/null 2>&1; then
    cd "$DELIVERY_DIR"
    sha256sum "$(basename "$ARCHIVE")" > "$(basename "$HASH_FILE")"
elif command -v shasum >/dev/null 2>&1; then
    cd "$DELIVERY_DIR"
    shasum -a 256 "$(basename "$ARCHIVE")" > "$(basename "$HASH_FILE")"
else
    printf '%s\n' 'erro: nenhuma ferramenta SHA-256 disponível' >&2
    exit 1
fi

printf '%s\n' "snapshot=$ARCHIVE"
printf '%s\n' "sha256=$HASH_FILE"
