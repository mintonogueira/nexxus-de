#!/bin/sh
# Auditoria conservadora contra bashisms nos scripts de orquestração Nexxus.
set -eu
SCRIPT_DIR=$(dirname "$0"); case "$SCRIPT_DIR" in -*) SCRIPT_DIR="./$SCRIPT_DIR" ;; esac
SCRIPT_DIR=$(CDPATH= cd "$SCRIPT_DIR" && pwd)
ROOT_DIR=$(CDPATH= cd "$SCRIPT_DIR/.." && pwd)

status=0
for file in "$ROOT_DIR"/scripts/*.sh "$ROOT_DIR"/scripts/lib/*.sh; do
    [ -f "$file" ] || continue
    first=$(sed -n '1p' "$file")
    [ "$first" = '#!/bin/sh' ] || { printf 'Shebang não POSIX: %s\n' "$file" >&2; status=1; }
    # Os padrões cobrem construções conhecidas que não pertencem ao Shell POSIX.
    if grep -En '\[\[|\]\]|<<<|<\(|>\(|(^|[[:space:]])source[[:space:]]|(^|[[:space:]])declare[[:space:]]|(^|[[:space:]])local[[:space:]]|\$\(\(' "$file" >/dev/null 2>&1; then
        printf 'Possível bashism em %s\n' "$file" >&2
        grep -En '\[\[|\]\]|<<<|<\(|>\(|(^|[[:space:]])source[[:space:]]|(^|[[:space:]])declare[[:space:]]|(^|[[:space:]])local[[:space:]]|\$\(\(' "$file" >&2 || :
        status=1
    fi
    sh -n "$file" || status=1
done
exit "$status"
