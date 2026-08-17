#!/bin/sh
# Verifica no grafo da própria Etapa 02 que nenhuma implementação gráfica
# concreta foi adicionada ao Window Manager Core.
set -eu

ROOT_DIR=$(CDPATH= cd "$(dirname "$0")/.." && pwd)
mkdir -p "$ROOT_DIR/.build"
cd "$ROOT_DIR"
cargo metadata --format-version 1 --no-deps > .build/metadata.json

if grep -Ei '"name":"(x11|x11rb|wayland|smithay|wlroots|xcb|xkbcommon)' .build/metadata.json; then
    printf '%s\n' 'erro: dependência concreta de backend detectada no Window Manager Core' >&2
    exit 1
fi
printf '%s\n' 'OK: grafo da Etapa 02 permanece agnóstico de X11/Wayland'
