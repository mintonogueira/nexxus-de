#!/bin/sh
# Build, teste, empacotamento e instalação nativa da Etapa 04 no Arch Linux.
set -eu

SCRIPT_DIR=$(dirname "$0"); case "$SCRIPT_DIR" in -*) SCRIPT_DIR="./$SCRIPT_DIR" ;; esac
SCRIPT_DIR=$(CDPATH= cd "$SCRIPT_DIR" && pwd)
ROOT_DIR=$(CDPATH= cd "$SCRIPT_DIR/.." && pwd)
. "$ROOT_DIR/manifests/etapa-04.conf"
. "$ROOT_DIR/scripts/lib/common.sh"

[ -r /etc/os-release ] || die '/etc/os-release ausente'
. /etc/os-release
[ "${ID:-}" = 'arch' ] || die 'este script deve ser executado somente no Arch Linux'
require_unprivileged_user
find_privilege_command
BUILD_DIR="$ROOT_DIR/.build/arch"; DIST_DIR="$ROOT_DIR/dist/arch"
reset_build_dir "$BUILD_DIR"; mkdir -p "$DIST_DIR"; LOG_FILE="$BUILD_DIR/build.log"; : > "$LOG_FILE"
validate_stage_tree

missing=''
for package in $ARCH_BUILD_PACKAGES; do
    if ! pacman -Q "$package" >/dev/null 2>&1; then missing="$missing $package"; fi
done
if [ "$missing" != '' ]; then
    log_msg "[deps] instalando dependências Arch:$missing"
    run_privileged pacman -S --needed --noconfirm $missing
fi
validate_rust_toolchain
cd "$ROOT_DIR"; build_and_test_workspace; prepare_staging

PKG_DIR="$BUILD_DIR/package"; mkdir -p "$PKG_DIR"
cp "$STAGING_DIR/usr/bin/nexxus-x11-backend-check" "$PKG_DIR/nexxus-x11-backend-check"
binary_sha=$(sha256_file "$PKG_DIR/nexxus-x11-backend-check")
sed -e "s/@BINARY_SHA256@/$binary_sha/" "$ROOT_DIR/packaging/arch/PKGBUILD.in" > "$PKG_DIR/PKGBUILD"
chown -R "$(id -u):$(id -g)" "$PKG_DIR"
(
    cd "$PKG_DIR"
    run_logged makepkg --force --noconfirm
)
set -- "$PKG_DIR"/nexxus-backend-x11-*.pkg.tar.*
[ -f "$1" ] || die 'makepkg não produziu pacote nexxus-backend-x11'
cp "$1" "$DIST_DIR/"; final_package="$DIST_DIR/$(basename "$1")"
pacman -Qp "$final_package" >/dev/null || die 'pacman rejeitou metadados'
contents_file="$BUILD_DIR/package-contents.txt"
pacman -Qlp "$final_package" > "$contents_file"
grep -q 'usr/bin/nexxus-x11-backend-check$' "$contents_file" || die 'binário ausente do pacote Arch'
log_msg "[install] instalando exatamente $final_package"
run_privileged pacman -U --noconfirm "$final_package"
command -v nexxus-x11-backend-check >/dev/null 2>&1 || die 'binário não encontrado após instalação'

# O teste pós-instalação executa o binário que veio do pacote, sob um X server
# isolado, provando que o artefato instalado consegue assumir o papel de WM.
start_test_xserver
if run_logged nexxus-x11-backend-check --check; then smoke_status=0; else smoke_status=$?; fi
stop_test_xserver
[ "$smoke_status" -eq 0 ] || die 'smoke test do pacote Arch instalado falhou'
log_msg "[status] pacote=$(basename "$final_package") sha256=$(sha256_file "$final_package")"
