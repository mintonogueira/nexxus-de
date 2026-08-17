#!/bin/sh
# Validação completa local da Etapa 08 sem instalar o pacote no host.
set -eu
SCRIPT_DIR=$(dirname "$0"); case "$SCRIPT_DIR" in -*) SCRIPT_DIR="./$SCRIPT_DIR" ;; esac
SCRIPT_DIR=$(CDPATH= cd "$SCRIPT_DIR" && pwd)
ROOT_DIR=$(CDPATH= cd "$SCRIPT_DIR/.." && pwd)
. "$ROOT_DIR/scripts/lib/common.sh"

require_unprivileged_user
validate_rust_toolchain
sh "$ROOT_DIR/scripts/check-posix.sh"
cd "$ROOT_DIR"
build_and_test_workspace
# Visual Assets não pode introduzir toolkit/runtime gráfico próprio.
if grep -R -Eiq '(^|[^A-Za-z])(gtk|qt5|qt6|electron)([^A-Za-z]|$)' "$ROOT_DIR/crates" "$ROOT_DIR/Cargo.toml"; then
    die 'dependência de toolkit proibida detectada na Etapa 08'
fi
log_msg '[status] validação local da Etapa 08 aprovada'
