#!/bin/sh
# Valida o crate da Etapa 02 e transforma warnings em falhas de qualidade.
# O rustfmt é aplicado antes da validação para produzir uma árvore normalizada;
# a CI publica essa normalização na branch da etapa quando houver diferença.
set -eu

[ "$(id -u)" -ne 0 ] || { printf '%s\n' 'erro: cargo/rustc não devem ser executados como root' >&2; exit 1; }
command -v cargo >/dev/null 2>&1 || { printf '%s\n' 'erro: cargo não encontrado' >&2; exit 127; }
command -v rustfmt >/dev/null 2>&1 || { printf '%s\n' 'erro: rustfmt não encontrado' >&2; exit 127; }
cargo clippy --version >/dev/null 2>&1 || { printf '%s\n' 'erro: cargo clippy indisponível' >&2; exit 127; }

cargo fmt --all
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps
