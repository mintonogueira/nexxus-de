#!/bin/sh
# Pipeline Debian da Etapa 14: autoprovisiona, compila, testa e prepara staging.
set -eu
SCRIPT_DIR=$(dirname "$0"); case "$SCRIPT_DIR" in -*) SCRIPT_DIR="./$SCRIPT_DIR" ;; esac
SCRIPT_DIR=$(CDPATH= cd "$SCRIPT_DIR" && pwd)
ROOT_DIR=$(CDPATH= cd "$SCRIPT_DIR/.." && pwd)
. "$SCRIPT_DIR/lib/common.sh"
. "$ROOT_DIR/manifests/etapa-14.conf"
BUILD_DIR="$ROOT_DIR/.build/debian"; DIST_DIR="$ROOT_DIR/dist/debian"; LOG_FILE=''

preflight_debian() {
    [ -r /etc/os-release ] || die '/etc/os-release não encontrado'
    ID=''; . /etc/os-release
    [ "${ID:-}" = 'debian' ] || die "script exclusivo para Debian (ID=${ID:-desconhecido})"
    require_unprivileged_user; validate_stage_tree
    command -v dpkg-query >/dev/null 2>&1 || die 'dpkg-query não encontrado'
    command -v apt-get >/dev/null 2>&1 || die 'apt-get não encontrado'
    find_privilege_command
}

install_missing_debian_build_deps() {
    _missing=''
    for _pkg in $DEBIAN_BUILD_PACKAGES; do
        dpkg-query -W -f='${Status}' "$_pkg" 2>/dev/null | grep -q 'ok installed' || _missing="$_missing $_pkg"
    done
    [ "$_missing" != '' ] || { log_msg '[deps] Debian já atendido'; return 0; }
    [ "$PRIVILEGE_CMD" != '' ] || die "dependências ausentes ($_missing) e sudo/doas indisponível"
    run_logged "$PRIVILEGE_CMD" apt-get update || die 'apt-get update falhou'
    run_logged "$PRIVILEGE_CMD" apt-get install -y --no-install-recommends $_missing || die 'apt-get install falhou'
}

preflight_debian
reset_build_dir "$BUILD_DIR"; LOG_DIR="$BUILD_DIR/logs"; mkdir -p "$LOG_DIR" "$DIST_DIR"
LOG_FILE="$LOG_DIR/build-$(date -u '+%Y%m%dT%H%M%SZ').log"; : > "$LOG_FILE"
install_missing_debian_build_deps
validate_rust_toolchain
build_and_test_workspace
prepare_staging
[ "$NEXXUS_INSTALLABLE" = '0' ] && { finish_non_installable_stage; exit 0; }
die 'payload instalável não está definido para a Etapa 14'
