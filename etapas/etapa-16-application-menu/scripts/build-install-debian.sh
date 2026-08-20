#!/bin/sh
# Pipeline Debian da Etapa 16: autoprovisiona, compila, testa e prepara staging.
set -eu
SCRIPT_DIR=$(CDPATH= cd "$(dirname "$0")" && pwd)
ROOT_DIR=$(CDPATH= cd "$SCRIPT_DIR/.." && pwd)
. "$ROOT_DIR/manifests/etapa-16.conf"
[ -r /etc/os-release ] || { printf '%s\n' 'ERRO: /etc/os-release ausente' >&2; exit 1; }
ID=''; . /etc/os-release
[ "${ID:-}" = 'debian' ] || { printf 'ERRO: script exclusivo para Debian (ID=%s)\n' "${ID:-desconhecido}" >&2; exit 1; }
[ "$(id -u)" -ne 0 ] || { printf '%s\n' 'ERRO: build não deve executar como root' >&2; exit 1; }
PRIV=''; command -v sudo >/dev/null 2>&1 && PRIV=sudo; [ -n "$PRIV" ] || { command -v doas >/dev/null 2>&1 && PRIV=doas || :; }
MISSING=''
for PKG in $DEBIAN_BUILD_PACKAGES; do dpkg-query -W -f='${Status}' "$PKG" 2>/dev/null | grep -q 'ok installed' || MISSING="$MISSING $PKG"; done
if [ -n "$MISSING" ]; then [ -n "$PRIV" ] || { printf 'ERRO: dependências ausentes:%s\n' "$MISSING" >&2; exit 1; }; $PRIV apt-get update; $PRIV apt-get install -y $MISSING; fi
cd "$ROOT_DIR"
cargo build --workspace --release
sh "$ROOT_DIR/scripts/check.sh"
rm -rf "$ROOT_DIR/.build/debian"
mkdir -p "$ROOT_DIR/.build/debian/staging"
printf '%s\n' "$NEXXUS_STAGE_ID" > "$ROOT_DIR/.build/debian/staging/.nexxus-stage"
[ "$NEXXUS_INSTALLABLE" = '0' ] && printf '%s\n' '[package/install] N/A: plugin integrável sem payload independente nesta etapa'
