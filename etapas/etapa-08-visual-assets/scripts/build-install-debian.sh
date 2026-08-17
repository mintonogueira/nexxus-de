#!/bin/sh
# Build, teste, empacotamento e instalação nativa da Etapa 08 no Debian.
set -eu
SCRIPT_DIR=$(dirname "$0"); case "$SCRIPT_DIR" in -*) SCRIPT_DIR="./$SCRIPT_DIR" ;; esac
SCRIPT_DIR=$(CDPATH= cd "$SCRIPT_DIR" && pwd)
ROOT_DIR=$(CDPATH= cd "$SCRIPT_DIR/.." && pwd)
. "$ROOT_DIR/manifests/etapa-08.conf"
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

missing=''
for package in $DEBIAN_BUILD_PACKAGES $DEBIAN_RUNTIME_PACKAGES; do
    if ! dpkg-query -W -f='${Status}\n' "$package" 2>/dev/null | grep -q 'install ok installed'; then missing="$missing $package"; fi
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

PKG_ROOT="$BUILD_DIR/pkgroot"
mkdir -p "$PKG_ROOT/DEBIAN"
cp -R "$STAGING_DIR/." "$PKG_ROOT/"
sed -e "s/@VERSION@/$NEXXUS_VERSION/g" "$ROOT_DIR/packaging/debian/control.in" > "$PKG_ROOT/DEBIAN/control"
final_package="$DIST_DIR/nexxus-visual-assets_${NEXXUS_VERSION}_all.deb"
dpkg-deb --build --root-owner-group "$PKG_ROOT" "$final_package"
dpkg-deb --info "$final_package" >/dev/null || die 'dpkg-deb rejeitou metadados'
dpkg-deb --contents "$final_package" > "$BUILD_DIR/package-contents.txt"
grep -q 'usr/share/nexxus/assets/manifest.toml$' "$BUILD_DIR/package-contents.txt" || die 'manifesto ausente do pacote Debian'
if grep -E '\.(ttf|otf|woff2?)$' "$BUILD_DIR/package-contents.txt" >/dev/null 2>&1; then die 'pacote Debian não pode vendorizar fontes'; fi
log_msg "[install] instalando exatamente $final_package"
run_privileged apt-get install -y "$final_package"
verify_installed_payload
printf 'debian_package=%s\ndebian_sha256=%s\n' "$(basename "$final_package")" "$(sha256_file "$final_package")" > "$ROOT_DIR/metrics/debian-package.txt"
log_msg "[status] pacote=$(basename "$final_package") sha256=$(sha256_file "$final_package")"
