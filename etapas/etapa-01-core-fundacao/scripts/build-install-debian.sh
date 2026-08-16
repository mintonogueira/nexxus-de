#!/bin/sh
# Pipeline Debian da Etapa 01: preflight, autoprovisionamento, build, testes e
# staging. Empacotamento/instalação só ocorrem quando o manifesto da etapa
# declarar um payload instalável.
set -eu

SCRIPT_DIR=$(dirname "$0")
case "$SCRIPT_DIR" in -*) SCRIPT_DIR="./$SCRIPT_DIR" ;; esac
SCRIPT_DIR=$(CDPATH= cd "$SCRIPT_DIR" && pwd)
ROOT_DIR=$(CDPATH= cd "$SCRIPT_DIR/.." && pwd)
. "$SCRIPT_DIR/lib/common.sh"
. "$ROOT_DIR/manifests/etapa-01.conf"

BUILD_DIR="$ROOT_DIR/.build/debian"
DIST_DIR="$ROOT_DIR/dist/debian"
LOG_FILE=''

# Valida Debian antes de executar APT ou alterar pacotes do host.
preflight_debian() {
    [ -r /etc/os-release ] || die "/etc/os-release não encontrado"
    ID=''
    . /etc/os-release
    [ "${ID:-}" = 'debian' ] || die "este script é exclusivo para Debian (ID=${ID:-desconhecido})"
    require_unprivileged_user
    validate_stage_tree
    command -v apt-get >/dev/null 2>&1 || die "apt-get não encontrado em host Debian"
    command -v dpkg-query >/dev/null 2>&1 || die "dpkg-query não encontrado em host Debian"
    find_privilege_command
    log_msg "[preflight] Debian detectado; usuário=$(id -un); arquitetura=$(uname -m)"
}

# Instala somente dependências de build/packaging realmente ausentes. O índice
# APT só é atualizado quando há algo a instalar.
install_missing_debian_build_deps() {
    _missing=''
    for _pkg in $DEBIAN_BUILD_PACKAGES; do
        if ! dpkg-query -W -f='${Status}\n' "$_pkg" 2>/dev/null | grep -q '^install ok installed$'; then
            _missing="$_missing $_pkg"
        fi
    done
    if [ "$_missing" = '' ]; then
        log_msg "[deps] dependências Debian já atendidas"
        return 0
    fi
    [ "$PRIVILEGE_CMD" != '' ] || die "dependências ausentes ($_missing) e sudo/doas indisponível"
    log_msg "[deps] atualizando índices APT porque existem dependências ausentes"
    run_logged "$PRIVILEGE_CMD" apt-get update || die "apt-get update falhou"
    log_msg "[deps] instalando somente dependências Debian ausentes:$_missing"
    # A lista vem do manifesto versionado da própria etapa; expansão por
    # palavras é intencional para passar os nomes de pacotes ao apt-get.
    run_logged "$PRIVILEGE_CMD" apt-get install -y --no-install-recommends $_missing || die "apt-get install falhou"
}

preflight_debian
reset_build_dir "$BUILD_DIR"
LOG_DIR="$BUILD_DIR/logs"
mkdir -p "$LOG_DIR" "$DIST_DIR"
LOG_FILE="$LOG_DIR/build-$(date -u '+%Y%m%dT%H%M%SZ').log"
: > "$LOG_FILE"
log_msg "[run] etapa=$NEXXUS_STAGE_ID versão=$NEXXUS_VERSION"
install_missing_debian_build_deps
validate_rust_toolchain
build_and_test_workspace
prepare_staging

if [ "$NEXXUS_INSTALLABLE" = '0' ]; then
    finish_non_installable_stage
    exit 0
fi

die "manifesto declarou payload instalável, mas a Etapa 01 não define driver de pacote Debian; isso deve ser implementado pela etapa que introduzir o payload"
