#!/bin/sh
# Gera snapshot versionável da Etapa 08 e registra seu SHA-256.
set -eu
SCRIPT_DIR=$(dirname "$0"); case "$SCRIPT_DIR" in -*) SCRIPT_DIR="./$SCRIPT_DIR" ;; esac
SCRIPT_DIR=$(CDPATH= cd "$SCRIPT_DIR" && pwd)
ROOT_DIR=$(CDPATH= cd "$SCRIPT_DIR/.." && pwd)
. "$ROOT_DIR/manifests/etapa-08.conf"
. "$ROOT_DIR/scripts/lib/common.sh"

OUT_DIR="$ROOT_DIR/entrega"
archive="$OUT_DIR/Nexxus_Etapa08_Visual_Assets_${NEXXUS_VERSION}.tar.gz"
mkdir -p "$OUT_DIR"
rm -f "$archive" "$archive.sha256"
cd "$ROOT_DIR"
set -- Cargo.toml Cargo.lock README.md STATUS.md assets crates docs manifests packaging scripts metrics
include=''
for item in "$@"; do [ -e "$item" ] && include="$include $item"; done
# O snapshot não inclui .build, target, dist nem a própria pasta entrega.
# shellcheck disable=SC2086
TZ=UTC tar --sort=name --mtime='UTC 2026-08-17' --owner=0 --group=0 --numeric-owner -czf "$archive" $include
hash=$(sha256_file "$archive")
printf '%s  %s\n' "$hash" "$(basename "$archive")" > "$archive.sha256"
log_msg "[delivery] $(basename "$archive") sha256=$hash"
