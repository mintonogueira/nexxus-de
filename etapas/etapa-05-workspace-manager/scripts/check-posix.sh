#!/bin/sh
# Rejeita bashisms conhecidos nos wrappers Shell próprios da Etapa 05.
set -eu

SCRIPT_DIR=$(dirname "$0"); case "$SCRIPT_DIR" in -*) SCRIPT_DIR="./$SCRIPT_DIR" ;; esac
ROOT_DIR=$(CDPATH= cd "$SCRIPT_DIR/.." && pwd)

find "$ROOT_DIR/scripts" -type f -name '*.sh' ! -name 'check-posix.sh' -print | while IFS= read -r file; do
    first=$(sed -n '1p' "$file")
    [ "$first" = '#!/bin/sh' ] || { printf '%s\n' "erro: shebang não POSIX em $file" >&2; exit 1; }
    if grep -En '\[\[|\]\]|<<<|<\(|>\(|(^|[;[:space:]])source[[:space:]]|(^|[;[:space:]])declare[[:space:]]|(^|[;[:space:]])mapfile[[:space:]]|(^|[;[:space:]])coproc([;[:space:]]|$)' "$file" >/dev/null 2>&1; then
        printf '%s\n' "erro: possível bashism em $file" >&2
        exit 1
    fi
done

printf '%s\n' 'auditoria POSIX: OK'
