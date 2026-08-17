#!/bin/sh
# Valida o Tiling Engine e trata warnings como falhas de qualidade.
set -eu

[ "$(id -u)" -ne 0 ] || { printf '%s\n' 'erro: cargo/rustc não devem ser executados como root' >&2; exit 1; }
command -v cargo >/dev/null 2>&1 || { printf '%s\n' 'erro: cargo não encontrado' >&2; exit 127; }
command -v rustfmt >/dev/null 2>&1 || { printf '%s\n' 'erro: rustfmt não encontrado' >&2; exit 127; }
cargo clippy --version >/dev/null 2>&1 || { printf '%s\n' 'erro: cargo clippy indisponível' >&2; exit 127; }

# Formata exclusivamente o crate da Etapa 06; dependências de etapas anteriores
# são somente leitura e não podem ser alteradas por esta conversa.
cargo fmt --package nexxus-tiling
cargo fmt --package nexxus-tiling -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --all-features --no-deps
