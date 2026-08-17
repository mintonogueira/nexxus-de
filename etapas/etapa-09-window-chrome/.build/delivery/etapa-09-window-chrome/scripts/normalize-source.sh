#!/bin/sh
# Normalização idempotente de fonte da Etapa 09 antes de rustfmt/clippy.
# O delivery executa a mesma rotina e versiona o resultado normalizado.
set -eu

SCRIPT_DIR=$(dirname "$0"); case "$SCRIPT_DIR" in -*) SCRIPT_DIR="./$SCRIPT_DIR" ;; esac
ROOT_DIR=$(CDPATH= cd "$SCRIPT_DIR/.." && pwd)
X11_FILE="$ROOT_DIR/crates/nexxus-window-chrome/src/x11.rs"
RENDER_FILE="$ROOT_DIR/crates/nexxus-window-chrome/src/render.rs"
UNIT_TEST="$ROOT_DIR/crates/nexxus-window-chrome/tests/window_chrome.rs"
X11_TEST="$ROOT_DIR/crates/nexxus-window-chrome/tests/x11_chrome.rs"
TMP_X11="$ROOT_DIR/.normalize-x11.$$"
TMP_RENDER="$ROOT_DIR/.normalize-render.$$"

sed 's/Atom, AtomEnum, ChangeWindowAttributesAux, ConfigureWindowAux/Atom, AtomEnum, ConfigureWindowAux/' "$X11_FILE" > "$TMP_X11"
mv "$TMP_X11" "$X11_FILE"

sed '/^        } else {$/ {
N
s/        } else {\n            if state.active { palette.surface_alt } else { palette.surface }/        } else if state.active {\
            palette.surface_alt\
        } else {\
            palette.surface/
}' "$RENDER_FILE" > "$TMP_RENDER"
mv "$TMP_RENDER" "$RENDER_FILE"

# No checkout do repositório, o catálogo da Etapa 08 parte de assets/icons;
# no sistema instalado, SYSTEM_ASSET_ROOT já aponta para a raiz final correta.
for test_file in "$UNIT_TEST" "$X11_TEST"; do
    tmp="$test_file.normalize.$$"
    sed 's#../etapa-08-visual-assets/assets"#../etapa-08-visual-assets/assets/icons"#g' "$test_file" > "$tmp"
    mv "$tmp" "$test_file"
done
