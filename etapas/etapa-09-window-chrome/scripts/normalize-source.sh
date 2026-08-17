#!/bin/sh
# Normalização idempotente de fonte da Etapa 09 antes de rustfmt/clippy.
# O delivery executa a mesma rotina e versiona o resultado normalizado.
set -eu

SCRIPT_DIR=$(dirname "$0"); case "$SCRIPT_DIR" in -*) SCRIPT_DIR="./$SCRIPT_DIR" ;; esac
ROOT_DIR=$(CDPATH= cd "$SCRIPT_DIR/.." && pwd)
FILE="$ROOT_DIR/crates/nexxus-window-chrome/src/x11.rs"
TMP="$ROOT_DIR/.normalize-x11.$$"

# Remove a importação residual que não participa do adapter atual. A substituição
# é exata e, por isso, não altera outras APIs ou contratos do módulo.
sed 's/Atom, AtomEnum, ChangeWindowAttributesAux, ConfigureWindowAux/Atom, AtomEnum, ConfigureWindowAux/' "$FILE" > "$TMP"
mv "$TMP" "$FILE"
