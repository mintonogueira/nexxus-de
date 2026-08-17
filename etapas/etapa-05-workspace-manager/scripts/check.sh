#!/bin/sh
# Valida o Workspace Manager e trata warnings como falhas de qualidade.
set -eu

[ "$(id -u)" -ne 0 ] || { printf '%s\n' 'erro: cargo/rustc não devem ser executados como root' >&2; exit 1; }
command -v cargo >/dev/null 2>&1 || { printf '%s\n' 'erro: cargo não encontrado' >&2; exit 127; }
command -v rustfmt >/dev/null 2>&1 || { printf '%s\n' 'erro: rustfmt não encontrado' >&2; exit 127; }
cargo clippy --version >/dev/null 2>&1 || { printf '%s\n' 'erro: cargo clippy indisponível' >&2; exit 127; }

cargo fmt --all
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --all-features --no-deps
