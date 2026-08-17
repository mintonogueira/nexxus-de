#!/bin/sh
# Validação completa da Etapa 10, incluindo grabs X11 reais em servidor Xvfb.
set -eu

SCRIPT_DIR=$(dirname "$0"); case "$SCRIPT_DIR" in -*) SCRIPT_DIR="./$SCRIPT_DIR" ;; esac
ROOT_DIR=$(CDPATH= cd "$SCRIPT_DIR/.." && pwd)
cd "$ROOT_DIR"

cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps

command -v Xvfb >/dev/null 2>&1 || {
    printf '%s\n' 'ERRO: Xvfb não encontrado para validação X11.' >&2
    exit 1
}

display_number=99
display=":$display_number"
socket="/tmp/.X11-unix/X$display_number"
xvfb_log="$ROOT_DIR/.build/xvfb.log"
mkdir -p "$ROOT_DIR/.build"
Xvfb "$display" -screen 0 1280x720x24 > "$xvfb_log" 2>&1 &
xvfb_pid=$!

cleanup_xvfb() {
    kill "$xvfb_pid" 2>/dev/null || :
    wait "$xvfb_pid" 2>/dev/null || :
}
trap cleanup_xvfb EXIT HUP INT TERM

ready=0
for _wait in 1 2 3 4 5 6 7 8 9 10; do
    if [ -S "$socket" ]; then ready=1; break; fi
    sleep 1
done
[ "$ready" -eq 1 ] || {
    cat "$xvfb_log" >&2
    printf '%s\n' 'ERRO: Xvfb não ficou pronto.' >&2
    exit 1
}

DISPLAY="$display" cargo test --workspace --test x11_grabs -- --ignored
printf '%s\n' '[ok] Shortcuts Core validado, incluindo X11/Xvfb.'
