#!/bin/sh
# Build, teste, empacotamento e instalação nativa da Etapa 03 no Debian.
set -eu

SCRIPT_DIR=$(dirname "$0"); case "$SCRIPT_DIR" in -*) SCRIPT_DIR="./$SCRIPT_DIR" ;; esac
SCRIPT_DIR=$(CDPATH= cd "$SCRIPT_DIR" && pwd)
ROOT_DIR=$(CDPATH= cd "$SCRIPT_DIR/.." && pwd)
. "$ROOT_DIR/manifests/etapa-03.conf"
. "$ROOT_DIR/scripts/lib/common.sh"

[ -r /etc/os-release ] || die '/etc/os-release ausente'
. /etc/os-release
[ "${ID:-}" = 'debian' ] || die 'este script deve ser executado somente no Debian'
require_unprivileged_user
find_privilege_command

BUILD_DIR="$ROOT_DIR/.build/debian"
DIST_DIR="$ROOT_DIR/dist/debian"
reset_build_dir "$BUILD_DIR"
mkdir -p "$DIST_DIR"
LOG_FILE="$BUILD_DIR/build.log"
: > "$LOG_FILE"

validate_stage_tree

missing=''
for package in $DEBIAN_BUILD_PACKAGES; do
    if ! dpkg-query -W -f='${Status}' "$package" 2>/dev/null | grep -q 'ok installed'; then
        missing="$missing $package"
    fi
done
if [ "$missing" != '' ]; then
    log_msg "[deps] instalando dependências Debian:$missing"
    run_privileged apt-get update
    run_privileged apt-get install -y --no-install-recommends $missing
fi
validate_rust_toolchain

cd "$ROOT_DIR"
build_and_test_workspace
prepare_staging

PKG_ROOT="$BUILD_DIR/package-root"
reset_build_dir "$PKG_ROOT"
mkdir -p "$PKG_ROOT/DEBIAN" "$PKG_ROOT/usr/bin" "$PKG_ROOT/usr/share/doc/nexxus-session"
cp "$STAGING_DIR/usr/bin/nexxus-session" "$PKG_ROOT/usr/bin/nexxus-session"
cp "$ROOT_DIR/config/session.toml.example" "$PKG_ROOT/usr/share/doc/nexxus-session/session.toml.example"
chmod 0755 "$PKG_ROOT/usr/bin/nexxus-session"
chmod 0644 "$PKG_ROOT/usr/share/doc/nexxus-session/session.toml.example"

arch=$(dpkg --print-architecture)
sed -e "s/@VERSION@/$NEXXUS_VERSION/" -e "s/@ARCH@/$arch/" \
    "$ROOT_DIR/packaging/debian/control.in" > "$PKG_ROOT/DEBIAN/control"
chmod 0644 "$PKG_ROOT/DEBIAN/control"

final_package="$DIST_DIR/nexxus-session_${NEXXUS_VERSION}_${arch}.deb"
log_msg '[package] gerando .deb por dpkg-deb'
dpkg-deb --root-owner-group --build "$PKG_ROOT" "$final_package" >/dev/null

log_msg '[validate] inspecionando metadados e payload Debian'
dpkg-deb --info "$final_package" >/dev/null || die 'dpkg-deb rejeitou metadados do pacote'
dpkg-deb --contents "$final_package" | grep -q './usr/bin/nexxus-session$' || die 'binário ausente do .deb'

log_msg "[install] instalando exatamente $final_package"
run_privileged apt-get install -y "$final_package"
command -v nexxus-session >/dev/null 2>&1 || die 'nexxus-session não encontrado após instalação'
log_msg "[status] pacote=$(basename "$final_package") sha256=$(sha256_file "$final_package")"
