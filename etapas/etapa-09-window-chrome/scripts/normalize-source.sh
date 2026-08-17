#!/bin/sh
# Normalização idempotente de fonte da Etapa 09 antes de rustfmt/clippy.
# O delivery executa a mesma rotina e versiona o resultado normalizado.
set -eu

SCRIPT_DIR=$(dirname "$0"); case "$SCRIPT_DIR" in -*) SCRIPT_DIR="./$SCRIPT_DIR" ;; esac
ROOT_DIR=$(CDPATH= cd "$SCRIPT_DIR/.." && pwd)
X11_FILE="$ROOT_DIR/crates/nexxus-window-chrome/src/x11.rs"
RENDER_FILE="$ROOT_DIR/crates/nexxus-window-chrome/src/render.rs"
TMP_X11="$ROOT_DIR/.normalize-x11.$$"
TMP_RENDER="$ROOT_DIR/.normalize-render.$$"

# Remove a importação residual que não participa do adapter atual.
sed 's/Atom, AtomEnum, ChangeWindowAttributesAux, ConfigureWindowAux/Atom, AtomEnum, ConfigureWindowAux/' "$X11_FILE" > "$TMP_X11"
mv "$TMP_X11" "$X11_FILE"

# Colapsa o único else/if identificado pelo Clippy no cálculo do fundo do botão.
# O padrão é exato para não modificar outros branches do renderer.
sed '/^        } else {$/ {
N
s/        } else {\n            if state.active { palette.surface_alt } else { palette.surface }/        } else if state.active {\
            palette.surface_alt\
        } else {\
            palette.surface/
}' "$RENDER_FILE" > "$TMP_RENDER"
mv "$TMP_RENDER" "$RENDER_FILE"
