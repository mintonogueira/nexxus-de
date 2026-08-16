#!/bin/sh
# Pipeline Arch Linux da Etapa 01: preflight, autoprovisionamento, build,
# testes e staging. Empacotamento/instalação só ocorrem quando o manifesto da
# etapa declarar um payload instalável.
set -eu

SCRIPT_DIR=$(dirname "$0")
case "$SCRIPT_DIR" in -*) SCRIPT_DIR="./$SCRIPT_DIR" ;; esac
SCRIPT_DIR=$(CDPATH= cd "$SCRIPT_DIR" && pwd)
ROOT_DIR=$(CDPATH= cd "$SCRIPT_DIR/.." && pwd)
. "$SCRIPT_DIR/lib/common.sh"
. "$ROOT_DIR/manifests/etapa-01.conf"

BUILD_DIR="$ROOT_DIR/.build/arch"
DIST_DIR="$ROOT_DIR/dist/arch"
LOG_FILE=''

# Valida a distribuição antes de qualquer mutação do sistema.
preflight_arch() {
    [ -r /etc/os-release ] || die "/etc/os-release não encontrado"
    ID=''
    . /etc/os-release
    [ "${ID:-}" = 'arch' ] || die "este script é exclusivo para Arch Linux (ID=${ID:-desconhecido})"
    require_unprivileged_user
    validate_stage_tree
    command -v pacman >/dev/null 2>&1 || die "pacman não encontrado em host Arch Linux"
    find_privilege_command
    log_msg "[preflight] Arch Linux detectado; usuário=$(id -un); arquitetura=$(uname -m)"
}

# Instala somente pacotes declarados no manifesto que não estejam presentes.
install_missing_arch_build_deps() {
    _missing=''
    for _pkg in $ARCH_BUILD_PACKAGES; do
        if ! pacman -Q "$_pkg" >/dev/null 2>&1; then
            _missing="$_missing $_pkg"
        fi
    done
    if [ "$_missing" = '' ]; then
        log_msg "[deps] dependências Arch já atendidas"
        return 0
    fi
    [ "$PRIVILEGE_CMD" != '' ] || die "dependências ausentes ($_missing) e sudo/doas indisponível"
    log_msg "[deps] instalando somente dependências Arch ausentes:$_missing"
    # A lista vem do manifesto versionado da própria etapa; expansão por
    # palavras é intencional para passar os nomes de pacotes ao pacman.
    run_logged "$PRIVILEGE_CMD" pacman -S --needed --noconfirm $_missing || die "pacman falhou ao instalar dependências"
}

preflight_arch
reset_build_dir "$BUILD_DIR"
LOG_DIR="$BUILD_DIR/logs"
mkdir -p "$LOG_DIR" "$DIST_DIR"
LOG_FILE="$LOG_DIR/build-$(date -u '+%Y%m%dT%H%M%SZ').log"
: > "$LOG_FILE"
log_msg "[run] etapa=$NEXXUS_STAGE_ID versão=$NEXXUS_VERSION"
install_missing_arch_build_deps
validate_rust_toolchain
build_and_test_workspace
prepare_staging

if [ "$NEXXUS_INSTALLABLE" = '0' ]; then
    finish_non_installable_stage
    exit 0
fi

die "manifesto declarou payload instalável, mas a Etapa 01 não define driver de pacote Arch; isso deve ser implementado pela etapa que introduzir o payload"
