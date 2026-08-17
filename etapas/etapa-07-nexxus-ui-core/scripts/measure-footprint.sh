#!/bin/sh
# Mede o pico de RSS do harness compilado, sem incluir Cargo no processo medido.
set -eu

SCRIPT_DIR=$(dirname "$0"); case "$SCRIPT_DIR" in -*) SCRIPT_DIR="./$SCRIPT_DIR" ;; esac
ROOT_DIR=$(CDPATH= cd "$SCRIPT_DIR/.." && pwd)
BUILD_DIR=${BUILD_DIR:-"$ROOT_DIR/.build/metrics"}
METRICS_DIR="$ROOT_DIR/metrics"
BINARY="$ROOT_DIR/target/release/nexxus-ui-demo"
OUTPUT="$BUILD_DIR/nexxus-ui-demo.ppm"
RESULT="$METRICS_DIR/footprint.txt"

[ -x "$BINARY" ] || { printf '%s\n' "erro: demo release ausente: $BINARY" >&2; exit 1; }
[ -x /usr/bin/time ] || { printf '%s\n' 'erro: /usr/bin/time é necessário para medir RSS' >&2; exit 1; }
mkdir -p "$BUILD_DIR" "$METRICS_DIR"

# GNU time está presente nos cenários Arch/Debian homologados da etapa e mede
# apenas o processo do harness, fornecendo uma linha estável e auditável.
/usr/bin/time -f 'max_rss_kib=%M\nelapsed_s=%e' -o "$RESULT" "$BINARY" "$OUTPUT" >/dev/null
printf 'binary_bytes=%s\n' "$(wc -c < "$BINARY" | tr -d ' ')" >> "$RESULT"
printf 'frame_bytes=%s\n' "$(wc -c < "$OUTPUT" | tr -d ' ')" >> "$RESULT"
printf '%s\n' "footprint=$RESULT"
cat "$RESULT"
