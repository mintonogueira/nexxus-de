#!/bin/sh
# Build, teste, empacotamento e instalação nativa da Etapa 04 no Debian.
set -eu

SCRIPT_DIR=$(dirname "$0"); case "$SCRIPT_DIR" in -*) SCRIPT_DIR="./$SCRIPT_DIR" ;; esac
SCRIPT_DIR=$(CDPATH= cd "$SCRIPT_DIR" && pwd)
ROOT_DIR=$(CDPATH= cd "$SCRIPT_DIR/.." && pwd)
. "$ROOT_DIR/manifests/etapa-04.conf"
. "$ROOT_DIR/scripts/lib/common.sh"

[ -r /etc/os-release ] || die '/etc/os-release ausente'
. /etc/os-release
[ "${ID:-}" = 'debian' ] || die 'este script deve ser executado somente no Debian'
require_unprivileged_user
find_privilege_command
BUILD_DIR="$ROOT_DIR/.build/debian"; DIST_DIR="$ROOT_DIR/dist/debian"
reset_build_dir "$BUILD_DIR"; mkdir -p "$DIST_DIR"; LOG_FILE="$BUILD_DIR/build.log"; : > "$LOG_FILE"
validate_stage_tree

missing=''
for package in $DEBIAN_BUILD_PACKAGES; do
    if ! dpkg-query -W -f='${Status}' "$package" 2>/dev/null | grep -q 'ok installed'; then missing="$missing $package"; fi
done
if [ "$missing" != '' ]; then
    log_msg "[deps] instalando dependências Debian:$missing"
    run_privileged apt-get update
    run_privileged apt-get install -y --no-install-recommends $missing
fi
validate_rust_toolchain
cd "$ROOT_DIR"; build_and_test_workspace; prepare_staging

PKG_ROOT="$BUILD_DIR/package-root"; reset_build_dir "$PKG_ROOT"
mkdir -p "$PKG_ROOT/DEBIAN" "$PKG_ROOT/usr/bin" "$PKG_ROOT/usr/share/doc/nexxus-backend-x11"
cp "$STAGING_DIR/usr/bin/nexxus-x11-backend-check" "$PKG_ROOT/usr/bin/"
cp "$ROOT_DIR/README.md" "$PKG_ROOT/usr/share/doc/nexxus-backend-x11/README.md"
arch=$(dpkg --print-architecture)
sed -e "s/@VERSION@/$NEXXUS_VERSION/" -e "s/@ARCH@/$arch/" "$ROOT_DIR/packaging/debian/control.in" > "$PKG_ROOT/DEBIAN/control"
final_package="$DIST_DIR/nexxus-backend-x11_${NEXXUS_VERSION}_${arch}.deb"
dpkg-deb --root-owner-group --build "$PKG_ROOT" "$final_package" >/dev/null
dpkg-deb --info "$final_package" >/dev/null || die 'dpkg-deb rejeitou metadados'
dpkg-deb --contents "$final_package" | grep -q './usr/bin/nexxus-x11-backend-check$' || die 'binário ausente do .deb'
log_msg "[install] instalando exatamente $final_package"
run_privileged apt-get install -y "$final_package"
command -v nexxus-x11-backend-check >/dev/null 2>&1 || die 'binário não encontrado após instalação'
log_msg "[status] pacote=$(basename "$final_package") sha256=$(sha256_file "$final_package")"
