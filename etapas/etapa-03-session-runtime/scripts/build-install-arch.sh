#!/bin/sh
# Build, teste, empacotamento e instalação nativa da Etapa 03 no Arch Linux.
set -eu

SCRIPT_DIR=$(dirname "$0"); case "$SCRIPT_DIR" in -*) SCRIPT_DIR="./$SCRIPT_DIR" ;; esac
SCRIPT_DIR=$(CDPATH= cd "$SCRIPT_DIR" && pwd)
ROOT_DIR=$(CDPATH= cd "$SCRIPT_DIR/.." && pwd)
. "$ROOT_DIR/manifests/etapa-03.conf"
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
LOG_FILE="$BUILD_DIR/build.log"
: > "$LOG_FILE"

validate_stage_tree

missing=''
for package in $ARCH_BUILD_PACKAGES; do
    if ! pacman -Q "$package" >/dev/null 2>&1; then
        missing="$missing $package"
    fi
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
cp "$STAGING_DIR/usr/bin/nexxus-session" "$PKG_DIR/nexxus-session"
cp "$ROOT_DIR/config/session.toml.example" "$PKG_DIR/session.toml.example"
binary_sha=$(sha256_file "$PKG_DIR/nexxus-session")
config_sha=$(sha256_file "$PKG_DIR/session.toml.example")
sed -e "s/@BINARY_SHA256@/$binary_sha/" -e "s/@CONFIG_SHA256@/$config_sha/" \
    "$ROOT_DIR/packaging/arch/PKGBUILD.in" > "$PKG_DIR/PKGBUILD"
chown -R "$(id -u):$(id -g)" "$PKG_DIR"

log_msg '[package] gerando pacote Arch por makepkg'
(
    cd "$PKG_DIR"
    run_logged makepkg --force --noconfirm
)
set -- "$PKG_DIR"/nexxus-session-*.pkg.tar.*
[ -f "$1" ] || die 'makepkg não produziu pacote nexxus-session'
package_file=$1
cp "$package_file" "$DIST_DIR/"
final_package="$DIST_DIR/$(basename "$package_file")"

log_msg '[validate] verificando pacote Arch'
pacman -Qp "$final_package" >/dev/null || die 'pacman rejeitou metadados do pacote gerado'
pacman -Qlp "$final_package" | grep -q 'usr/bin/nexxus-session$' || die 'binário ausente do pacote Arch'

log_msg "[install] instalando exatamente $final_package"
run_privileged pacman -U --noconfirm "$final_package"
command -v nexxus-session >/dev/null 2>&1 || die 'nexxus-session não encontrado após instalação'
log_msg "[status] pacote=$(basename "$final_package") sha256=$(sha256_file "$final_package")"
