#!/bin/sh
# Validação da Etapa 09: normaliza Rust, valida fronteiras e testa X11 real sob Xvfb.
set -eu

SCRIPT_DIR=$(dirname "$0"); case "$SCRIPT_DIR" in -*) SCRIPT_DIR="./$SCRIPT_DIR" ;; esac
ROOT_DIR=$(CDPATH= cd "$SCRIPT_DIR/.." && pwd)
cd "$ROOT_DIR"

# A normalização acontece antes do check para que o snapshot/commit final da
# etapa contenha exatamente o formato canônico produzido pelo rustfmt/clippy.
sh ./scripts/normalize-source.sh
cargo fmt --all
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features --test window_chrome

command -v xvfb-run >/dev/null 2>&1 || { printf '%s\n' 'ERRO: xvfb-run não encontrado' >&2; exit 1; }
xvfb-run -a -s '-screen 0 1920x1080x24' cargo test --workspace --all-features --test x11_chrome -- --test-threads=1

RUSTDOCFLAGS='-D warnings' cargo doc --workspace --all-features --no-deps

# A UI própria não pode introduzir toolkits externos. A string GTK é permitida
# somente no nome do atom de interoperabilidade usado para reconhecer CSD.
if grep -R -n -E '(^|[^[:alnum:]_])(gtk4|gtk::|qt5|qt6|qwidget|qwindow)([^[:alnum:]_]|$)' crates --include='*.rs'; then
    printf '%s\n' 'ERRO: toolkit externo detectado no Window Chrome' >&2
    exit 1
fi

# Wayland decorations finais e minimizar globalmente estão fora da Etapa 09.
if grep -R -n -E 'RequestMinimize|ChromeButton::Minimize|fn[[:space:]]+minimize' crates --include='*.rs'; then
    printf '%s\n' 'ERRO: minimizar globalmente foi introduzido fora do escopo' >&2
    exit 1
fi

printf '%s\n' '[ok] Etapa 09 validada: Rust + CSD/SSD + X11/Xvfb + fronteiras.'
