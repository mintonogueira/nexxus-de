#!/bin/sh
# Helpers POSIX compartilhados pelos cenários Arch e Debian da Etapa 04.

log_msg() {
    printf '%s\n' "$*"
    if [ "${LOG_FILE:-}" != '' ]; then printf '%s\n' "$*" >> "$LOG_FILE"; fi
}

die() { log_msg "ERRO: $*"; exit 1; }

require_unprivileged_user() {
    [ "$(id -u)" -ne 0 ] || die 'o pipeline de build do Nexxus não pode ser executado como root'
}

find_privilege_command() {
    if command -v sudo >/dev/null 2>&1; then PRIVILEGE_CMD='sudo'; elif command -v doas >/dev/null 2>&1; then PRIVILEGE_CMD='doas'; else PRIVILEGE_CMD=''; fi
}

run_privileged() {
    [ "$PRIVILEGE_CMD" != '' ] || die 'sudo/doas não encontrado para operação privilegiada'
    "$PRIVILEGE_CMD" "$@"
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
    [ -f "$ROOT_DIR/Cargo.toml" ] || die 'Cargo.toml da Etapa 04 não encontrado'
    [ -f "$ROOT_DIR/manifests/etapa-04.conf" ] || die 'manifesto da Etapa 04 não encontrado'
    [ -f "$ROOT_DIR/../etapa-01-core-fundacao/crates/nexxus-core/Cargo.toml" ] || die 'dependência nexxus-core ausente'
    [ -f "$ROOT_DIR/../etapa-01-core-fundacao/crates/nexxus-backend-api/Cargo.toml" ] || die 'dependência nexxus-backend-api ausente'
    [ -f "$ROOT_DIR/../etapa-02-window-manager-core/crates/nexxus-wm/Cargo.toml" ] || die 'dependência nexxus-wm ausente'
    [ -f "$ROOT_DIR/../etapa-03-session-runtime/crates/nexxus-session/Cargo.toml" ] || die 'dependência nexxus-session ausente'
}

validate_rust_toolchain() {
    command -v cargo >/dev/null 2>&1 || die 'cargo não encontrado após resolução de dependências'
    command -v rustc >/dev/null 2>&1 || die 'rustc não encontrado após resolução de dependências'
    command -v rustfmt >/dev/null 2>&1 || die 'rustfmt não encontrado após resolução de dependências'
    cargo clippy --version >/dev/null 2>&1 || die 'cargo clippy não encontrado após resolução de dependências'
}

start_test_xserver() {
    DISPLAY_NUMBER=${NEXXUS_XVFB_DISPLAY:-:97}
    Xvfb "$DISPLAY_NUMBER" -screen 0 1280x720x24 -nolisten tcp > "$BUILD_DIR/xvfb.log" 2>&1 &
    XVFB_PID=$!
    export DISPLAY="$DISPLAY_NUMBER"
    _attempt=0
    while [ "$_attempt" -lt 50 ]; do
        if xdpyinfo >/dev/null 2>&1; then return 0; fi
        _attempt=$((_attempt + 1))
        sleep 0.1
    done
    kill "$XVFB_PID" >/dev/null 2>&1 || :
    die 'Xvfb não ficou disponível para os testes'
}

stop_test_xserver() {
    if [ "${XVFB_PID:-}" != '' ]; then kill "$XVFB_PID" >/dev/null 2>&1 || :; wait "$XVFB_PID" 2>/dev/null || :; XVFB_PID=''; fi
}

build_and_test_workspace() {
    log_msg '[build] compilando Backend X11 em release'
    run_logged cargo build --workspace --release || die 'cargo build falhou'
    log_msg '[test] iniciando Xvfb e executando fmt, clippy, testes e rustdoc'
    start_test_xserver
    if run_logged sh "$ROOT_DIR/scripts/check.sh"; then _test_status=0; else _test_status=$?; fi
    stop_test_xserver
    [ "$_test_status" -eq 0 ] || die 'validação Rust/X11 falhou'
}

prepare_staging() {
    STAGING_DIR="$BUILD_DIR/staging"
    reset_build_dir "$STAGING_DIR"
    mkdir -p "$STAGING_DIR/usr/bin" "$STAGING_DIR/usr/share/doc/nexxus-backend-x11"
    cp "$ROOT_DIR/target/release/nexxus-x11-backend-check" "$STAGING_DIR/usr/bin/nexxus-x11-backend-check"
    chmod 0755 "$STAGING_DIR/usr/bin/nexxus-x11-backend-check"
    cp "$ROOT_DIR/README.md" "$STAGING_DIR/usr/share/doc/nexxus-backend-x11/README.md"
    chmod 0644 "$STAGING_DIR/usr/share/doc/nexxus-backend-x11/README.md"
}

sha256_file() { sha256sum "$1" | awk '{print $1}'; }
