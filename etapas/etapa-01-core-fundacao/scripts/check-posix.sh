#!/bin/sh
# Auditoria estática dos wrappers. Bloqueia bashisms expressamente proibidos
# pelo Aditivo 05 antes de qualquer build ou handoff.
set -eu

SCRIPT_DIR=$(dirname "$0")
case "$SCRIPT_DIR" in -*) SCRIPT_DIR="./$SCRIPT_DIR" ;; esac
SCRIPT_DIR=$(CDPATH= cd "$SCRIPT_DIR" && pwd)
ROOT_DIR=$(CDPATH= cd "$SCRIPT_DIR/.." && pwd)
FILES="$ROOT_DIR/scripts/build-install-arch.sh $ROOT_DIR/scripts/build-install-debian.sh $ROOT_DIR/scripts/check.sh $ROOT_DIR/scripts/check-posix.sh $ROOT_DIR/scripts/lib/common.sh"
AUDIT_FILES="$ROOT_DIR/scripts/build-install-arch.sh $ROOT_DIR/scripts/build-install-debian.sh $ROOT_DIR/scripts/check.sh $ROOT_DIR/scripts/lib/common.sh"

sh -n "$ROOT_DIR/manifests/etapa-01.conf"
for file in $FILES; do
    sh -n "$file"
    [ "$(sed -n '1p' "$file")" = '#!/bin/sh' ] || {
        printf '%s\n' "erro: shebang não POSIX em $file" >&2
        exit 1
    }
done

# O próprio auditor contém os padrões como dados, portanto não se autoanalisa.
if grep -nE '\[\[|\]\]|(^|[[:space:]])source[[:space:]]|<<<|<\(|>\(|(^|[[:space:]])function[[:space:]]|(^|[[:space:]])declare[[:space:]]|(^|[[:space:]])mapfile[[:space:]]|(^|[[:space:]])coproc([[:space:]]|$)' $AUDIT_FILES; then
    printf '%s\n' 'erro: extensão não POSIX encontrada nos wrappers' >&2
    exit 1
fi
printf '%s\n' 'OK: wrappers passaram na auditoria POSIX estática'
