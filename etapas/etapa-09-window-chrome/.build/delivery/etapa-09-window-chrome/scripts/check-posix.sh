#!/bin/sh
# Auditoria leve contra bashisms proibidos pelos Aditivos 05/07.
set -eu

SCRIPT_DIR=$(dirname "$0"); case "$SCRIPT_DIR" in -*) SCRIPT_DIR="./$SCRIPT_DIR" ;; esac
ROOT_DIR=$(CDPATH= cd "$SCRIPT_DIR/.." && pwd)
status=0

# Os próprios scripts de auditoria contêm expressões literais usadas para
# detectar bashisms; auditar essas expressões contra si mesmas gera falso
# positivo. Os wrappers operacionais continuam cobertos integralmente.
for file in \
    "$ROOT_DIR/scripts/build-install-arch.sh" \
    "$ROOT_DIR/scripts/build-install-debian.sh" \
    "$ROOT_DIR/scripts/create-delivery.sh" \
    "$ROOT_DIR/scripts/lib/common.sh"
do
    [ -f "$file" ] || continue
    first=$(sed -n '1p' "$file")
    [ "$first" = '#!/bin/sh' ] || { printf '%s: shebang não POSIX\n' "$file" >&2; status=1; }
    if grep -n -E '\[\[|\]\]|<<<|<\(|>\(|(^|[[:space:]])source[[:space:]]|(^|[[:space:]])declare[[:space:]]|(^|[[:space:]])local[[:space:]]|\$RANDOM|\$\{![^}]*\}' "$file"; then
        printf '%s: construção não POSIX detectada\n' "$file" >&2
        status=1
    fi
done

[ "$status" -eq 0 ] || exit "$status"
printf '%s\n' '[ok] wrappers operacionais da Etapa 09 compatíveis com /bin/sh POSIX.'
