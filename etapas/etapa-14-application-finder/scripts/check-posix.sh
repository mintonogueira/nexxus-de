#!/bin/sh
# Auditoria simples contra construções Shell sabidamente não POSIX.
set -eu
SCRIPT_DIR=$(dirname "$0"); case "$SCRIPT_DIR" in -*) SCRIPT_DIR="./$SCRIPT_DIR" ;; esac
ROOT_DIR=$(CDPATH= cd "$SCRIPT_DIR/.." && pwd)

for file in "$ROOT_DIR"/scripts/*.sh "$ROOT_DIR"/scripts/lib/*.sh; do
    [ -f "$file" ] || continue
    [ "$(basename "$file")" = 'check-posix.sh' ] && continue
    first=$(sed -n '1p' "$file")
    [ "$first" = '#!/bin/sh' ] || {
        printf 'ERRO: shebang não POSIX em %s\n' "$file" >&2
        exit 1
    }
    if grep -nE '\[\[|<<<|<\(|>\(|(^|[[:space:]])source[[:space:]]|(^|[[:space:]])declare[[:space:]]|(^|[[:space:]])mapfile[[:space:]]' "$file"; then
        printf 'ERRO: construção não POSIX detectada em %s\n' "$file" >&2
        exit 1
    fi
done
printf '%s\n' 'Auditoria POSIX aprovada.'
