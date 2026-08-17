#!/bin/sh
# Pipeline Arch da Etapa 05: autoprovisiona dependências, compila, testa e
# prepara staging sem fabricar pacote para uma biblioteca interna.
set -eu

SCRIPT_DIR=$(dirname "$0"); case "$SCRIPT_DIR" in -*) SCRIPT_DIR="./$SCRIPT_DIR" ;; esac
SCRIPT_DIR=$(CDPATH= cd "$SCRIPT_DIR" && pwd)
ROOT_DIR=$(CDPATH= cd "$SCRIPT_DIR/.." && pwd)
. "$SCRIPT_DIR/lib/common.sh"
. "$ROOT_DIR/manifests/etapa-05.conf"
BUILD_DIR="$ROOT_DIR/.build/arch"; DIST_DIR="$ROOT_DIR/dist/arch"; LOG_FILE=''

preflight_arch() {
    [ -r /etc/os-release ] || die '/etc/os-release não encontrado'
    ID=''; . /etc/os-release
    [ "${ID:-}" = 'arch' ] || die "script exclusivo para Arch Linux (ID=${ID:-desconhecido})"
    require_unprivileged_user; validate_stage_tree
    command -v pacman >/dev/null 2>&1 || die 'pacman não encontrado'
    find_privilege_command
}

install_missing_arch_build_deps() {
    _missing=''
    for _pkg in $ARCH_BUILD_PACKAGES; do
        pacman -Q "$_pkg" >/dev/null 2>&1 || _missing="$_missing $_pkg"
    done
    [ "$_missing" != '' ] || { log_msg '[deps] Arch já atendido'; return 0; }
    [ "$PRIVILEGE_CMD" != '' ] || die "dependências ausentes ($_missing) e sudo/doas indisponível"
    run_logged "$PRIVILEGE_CMD" pacman -S --needed --noconfirm $_missing || die 'pacman falhou'
}

preflight_arch
reset_build_dir "$BUILD_DIR"; LOG_DIR="$BUILD_DIR/logs"; mkdir -p "$LOG_DIR" "$DIST_DIR"
LOG_FILE="$LOG_DIR/build-$(date -u '+%Y%m%dT%H%M%SZ').log"; : > "$LOG_FILE"
install_missing_arch_build_deps
validate_rust_toolchain
build_and_test_workspace
prepare_staging
[ "$NEXXUS_INSTALLABLE" = '0' ] && { finish_non_installable_stage; exit 0; }
die 'payload instalável não está definido para a Etapa 05'
