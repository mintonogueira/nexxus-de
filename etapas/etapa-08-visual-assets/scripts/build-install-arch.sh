#!/bin/sh
# Build, teste, empacotamento e instalação nativa da Etapa 08 no Arch Linux.
set -eu
SCRIPT_DIR=$(dirname "$0"); case "$SCRIPT_DIR" in -*) SCRIPT_DIR="./$SCRIPT_DIR" ;; esac
SCRIPT_DIR=$(CDPATH= cd "$SCRIPT_DIR" && pwd)
ROOT_DIR=$(CDPATH= cd "$SCRIPT_DIR/.." && pwd)
. "$ROOT_DIR/manifests/etapa-08.conf"
. "$ROOT_DIR/scripts/lib/common.sh"

[ -r /etc/os-release ] || die '/etc/os-release ausente'
. /etc/os-release
[ "${ID:-}" = 'arch' ] || die 'este script deve ser executado somente no Arch Linux'
require_unprivileged_user
find_privilege_command
BUILD_DIR="$ROOT_DIR/.build/arch"
DIST_DIR="$ROOT_DIR/dist/arch"
reset_build_dir "$BUILD_DIR"
mkdir -p "$DIST_DIR"

missing=''
for package in $ARCH_BUILD_PACKAGES $ARCH_RUNTIME_PACKAGES; do
    if ! pacman -Q "$package" >/dev/null 2>&1; then missing="$missing $package"; fi
done
if [ "$missing" != '' ]; then
    log_msg "[deps] instalando dependências Arch:$missing"
    run_privileged pacman -S --needed --noconfirm $missing
fi
validate_rust_toolchain
cd "$ROOT_DIR"
build_and_test_workspace
prepare_staging

PKG_DIR="$BUILD_DIR/package"
mkdir -p "$PKG_DIR"
(
    cd "$STAGING_DIR"
    tar -czf "$PKG_DIR/nexxus-visual-assets-payload.tar.gz" .
)
payload_sha=$(sha256_file "$PKG_DIR/nexxus-visual-assets-payload.tar.gz")
sed -e "s/@VERSION@/$NEXXUS_VERSION/g" -e "s/@PAYLOAD_SHA256@/$payload_sha/g" "$ROOT_DIR/packaging/arch/PKGBUILD.in" > "$PKG_DIR/PKGBUILD"
(
    cd "$PKG_DIR"
    makepkg --force --noconfirm
)
set -- "$PKG_DIR"/nexxus-visual-assets-*.pkg.tar.*
[ -f "$1" ] || die 'makepkg não produziu pacote nexxus-visual-assets'
cp "$1" "$DIST_DIR/"
final_package="$DIST_DIR/$(basename "$1")"
pacman -Qp "$final_package" >/dev/null || die 'pacman rejeitou metadados do pacote'
pacman -Qlp "$final_package" > "$BUILD_DIR/package-contents.txt"
grep -q 'usr/share/nexxus/assets/manifest.toml$' "$BUILD_DIR/package-contents.txt" || die 'manifesto ausente do pacote Arch'
if grep -E '\.(ttf|otf|woff2?)$' "$BUILD_DIR/package-contents.txt" >/dev/null 2>&1; then die 'pacote Arch não pode vendorizar fontes'; fi
log_msg "[install] instalando exatamente $final_package"
run_privileged pacman -U --needed --noconfirm "$final_package"
verify_installed_payload
printf 'arch_package=%s\narch_sha256=%s\n' "$(basename "$final_package")" "$(sha256_file "$final_package")" > "$ROOT_DIR/metrics/arch-package.txt"
log_msg "[status] pacote=$(basename "$final_package") sha256=$(sha256_file "$final_package")"
