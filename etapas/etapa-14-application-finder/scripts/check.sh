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
# transitivamente. `cargo tree --prefix none` imprime "crate versão"; o espaço
# literal mantém esta checagem compatível com o auditor POSIX da própria etapa.
_tree=$(cargo tree -p nexxus-app-finder --prefix none)
for _crate in gtk gtk4 gtk-sys gtk4-sys qt5 qt6 qmetaobject cxx-qt; do
    if printf '%s\n' "$_tree" | grep "^${_crate} " >/dev/null 2>&1; then
        printf 'ERRO: dependência GTK/Qt detectada: %s\n' "$_crate" >&2
        exit 1
    fi
done
