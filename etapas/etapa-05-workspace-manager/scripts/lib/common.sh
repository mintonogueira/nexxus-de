#!/bin/sh
# Helpers POSIX compartilhados pelos cenários Arch e Debian da Etapa 05.

log_msg() {
    printf '%s\n' "$*"
    if [ "${LOG_FILE:-}" != '' ]; then
        printf '%s\n' "$*" >> "$LOG_FILE"
    fi
}

die() {
    log_msg "ERRO: $*"
    exit 1
}

require_unprivileged_user() {
    [ "$(id -u)" -ne 0 ] || die 'o pipeline de build do Nexxus não pode ser executado como root'
}

find_privilege_command() {
    if command -v sudo >/dev/null 2>&1; then
        PRIVILEGE_CMD='sudo'
    elif command -v doas >/dev/null 2>&1; then
        PRIVILEGE_CMD='doas'
    else
        PRIVILEGE_CMD=''
    fi
}

run_logged() {
    _run_output="${BUILD_DIR}/.command.$$.log"
    : > "$_run_output"
    if "$@" > "$_run_output" 2>&1; then _run_status=0; else _run_status=$?; fi
    cat "$_run_output"
    cat "$_run_output" >> "$LOG_FILE"
    rm -f "$_run_output"
    [ "$_run_status" -eq 0 ] || return "$_run_status"
}

reset_build_dir() {
    _reset_path=$1
    [ "$_reset_path" != '' ] || die 'caminho de build vazio'
    case "$_reset_path" in "$ROOT_DIR"/.build/*) ;; *) die "recusa limpar caminho fora de $ROOT_DIR/.build: $_reset_path" ;; esac
    rm -rf "$_reset_path"
    mkdir -p "$_reset_path"
}

validate_stage_tree() {
    [ -f "$ROOT_DIR/Cargo.toml" ] || die 'Cargo.toml da Etapa 05 não encontrado'
    [ -f "$ROOT_DIR/manifests/etapa-05.conf" ] || die 'manifesto da Etapa 05 não encontrado'
    [ -f "$ROOT_DIR/../etapa-01-core-fundacao/crates/nexxus-config/Cargo.toml" ] || die 'dependência nexxus-config ausente'
    [ -f "$ROOT_DIR/../etapa-02-window-manager-core/crates/nexxus-wm/Cargo.toml" ] || die 'dependência nexxus-wm ausente'
}

validate_rust_toolchain() {
    command -v cargo >/dev/null 2>&1 || die 'cargo não encontrado após resolução de dependências'
    command -v rustc >/dev/null 2>&1 || die 'rustc não encontrado após resolução de dependências'
    command -v rustfmt >/dev/null 2>&1 || die 'rustfmt não encontrado após resolução de dependências'
    cargo clippy --version >/dev/null 2>&1 || die 'cargo clippy não encontrado após resolução de dependências'
}

build_and_test_workspace() {
    log_msg '[build] compilando Workspace Manager em release'
    run_logged cargo build --workspace --release || die 'cargo build falhou'
    log_msg '[test] executando fmt, clippy, testes, rustdoc e neutralidade de backend'
    run_logged sh "$ROOT_DIR/scripts/check.sh" || die 'validação Rust falhou'
    run_logged sh "$ROOT_DIR/scripts/check-neutrality.sh" || die 'neutralidade de backend falhou'
}

prepare_staging() {
    STAGING_DIR="$BUILD_DIR/staging"
    reset_build_dir "$STAGING_DIR"
    printf '%s\n' "$NEXXUS_STAGE_ID" > "$STAGING_DIR/.nexxus-stage"
    log_msg "[staging] preparado em $STAGING_DIR"
}

finish_non_installable_stage() {
    log_msg '[package] N/A: NEXXUS_INSTALLABLE=0'
    log_msg '[install] N/A: biblioteca interna sem payload runtime independente'
    log_msg '[status] build/test/staging concluídos; pacote nativo e instalação permanecem N/A'
}
