#!/bin/sh
# Auditoria conservadora de bashisms nos scripts POSIX da Etapa 11.
set -eu
SCRIPT_DIR=$(dirname "$0"); case "$SCRIPT_DIR" in -*) SCRIPT_DIR="./$SCRIPT_DIR" ;; esac
SCRIPT_DIR=$(CDPATH= cd "$SCRIPT_DIR" && pwd)

status=0
for file in "$SCRIPT_DIR"/*.sh "$SCRIPT_DIR"/lib/*.sh; do
    [ -f "$file" ] || continue
    first=$(sed -n '1p' "$file")
    [ "$first" = '#!/bin/sh' ] || { printf 'shebang não POSIX: %s\n' "$file" >&2; status=1; }
    # Remove comentários antes da busca para não auto-detectar a lista abaixo.
    code=$(sed 's/[[:space:]]*#.*$//' "$file")
    printf '%s\n' "$code" | grep -E '\[\[|\]\]|<<<|<\(|>\(|(^|[[:space:]])source[[:space:]]|(^|[[:space:]])declare[[:space:]]|(^|[[:space:]])mapfile[[:space:]]|(^|[[:space:]])coproc[[:space:]]' >/dev/null 2>&1 && {
        printf 'possível bashism: %s\n' "$file" >&2
        status=1
    }
done
exit "$status"
