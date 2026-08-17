#!/bin/sh
# Validação da Etapa 14 sem modificar módulos predecessores nem o código-fonte.
set -eu
SCRIPT_DIR=$(dirname "$0"); case "$SCRIPT_DIR" in -*) SCRIPT_DIR="./$SCRIPT_DIR" ;; esac
ROOT_DIR=$(CDPATH= cd "$SCRIPT_DIR/.." && pwd)
cd "$ROOT_DIR"

# A validação não deve formatar silenciosamente o código: divergência de rustfmt
# é defeito e precisa falhar antes do handoff.
cargo fmt -p nexxus-app-finder -- --check
cargo clippy -p nexxus-app-finder --all-targets -- -D warnings
cargo test -p nexxus-app-finder
RUSTDOCFLAGS='-D warnings' cargo doc -p nexxus-app-finder --no-deps

# O Finder próprio do Nexxus não pode introduzir GTK/Qt direta ou
# transitivamente. A expressão é deliberadamente restrita a crates/toolkits
# conhecidos para evitar falsos positivos em nomes comuns.
if cargo tree -p nexxus-app-finder --prefix none | \
    grep -E '^(gtk|gtk4|gtk-sys|gtk4-sys|qt5|qt6|qmetaobject|cxx-qt)([[:space:]]|$)' >/dev/null 2>&1; then
    printf '%s\n' 'ERRO: dependência GTK/Qt detectada no Application Finder' >&2
    exit 1
fi
