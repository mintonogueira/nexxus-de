#!/bin/sh
# Validação Rust comum da Etapa 01. O script falha cedo se for executado como
# root para preservar a regra de build normal sob usuário não privilegiado.
set -eu

[ "$(id -u)" -ne 0 ] || {
    printf '%s\n' "erro: cargo/rustc não devem ser executados como root" >&2
    exit 1
}
command -v cargo >/dev/null 2>&1 || {
    printf '%s\n' "erro: cargo não encontrado no PATH" >&2
    exit 127
}
command -v rustfmt >/dev/null 2>&1 || {
    printf '%s\n' "erro: rustfmt não encontrado no PATH" >&2
    exit 127
}
# O contrato portátil é o subcomando Cargo; a presença direta de
# `clippy-driver` varia entre empacotamentos das distribuições.
cargo clippy --version >/dev/null 2>&1 || {
    printf '%s\n' "erro: cargo clippy não está disponível" >&2
    exit 127
}

cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps
