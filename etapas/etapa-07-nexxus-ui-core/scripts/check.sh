#!/bin/sh
# Valida o Nexxus UI Core e trata warnings como falhas de qualidade.
set -eu

SCRIPT_DIR=$(dirname "$0"); case "$SCRIPT_DIR" in -*) SCRIPT_DIR="./$SCRIPT_DIR" ;; esac
ROOT_DIR=$(CDPATH= cd "$SCRIPT_DIR/.." && pwd)

[ "$(id -u)" -ne 0 ] || { printf '%s\n' 'erro: cargo/rustc não devem ser executados como root' >&2; exit 1; }
command -v cargo >/dev/null 2>&1 || { printf '%s\n' 'erro: cargo não encontrado' >&2; exit 127; }
command -v rustfmt >/dev/null 2>&1 || { printf '%s\n' 'erro: rustfmt não encontrado' >&2; exit 127; }
cargo clippy --version >/dev/null 2>&1 || { printf '%s\n' 'erro: cargo clippy indisponível' >&2; exit 127; }

# Formata somente o crate pertencente à Etapa 07; outras etapas permanecem
# fora da fronteira de escrita desta conversa.
cargo fmt --package nexxus-ui
cargo fmt --package nexxus-ui -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --all-features --no-deps
sh "$ROOT_DIR/scripts/check-boundaries.sh"
