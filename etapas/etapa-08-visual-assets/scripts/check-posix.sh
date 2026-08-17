#!/bin/sh
# Rejeita bashisms conhecidos nos wrappers Shell próprios da Etapa 08.
set -eu

SCRIPT_DIR=$(dirname "$0"); case "$SCRIPT_DIR" in -*) SCRIPT_DIR="./$SCRIPT_DIR" ;; esac
ROOT_DIR=$(CDPATH= cd "$SCRIPT_DIR/.." && pwd)
TMP_FILE="${TMPDIR:-/tmp}/nexxus-assets-posix.$$.txt"
trap 'rm -f "$TMP_FILE"' 0 1 2 3 15

# O próprio auditor contém os padrões proibidos como dados e, por isso, não
# deve auditar a si mesmo. Strings simples são removidas antes da inspeção para
# evitar falsos positivos em mensagens, regexes e comentários técnicos.
find "$ROOT_DIR/scripts" -type f -name '*.sh' ! -name 'check-posix.sh' -print | while IFS= read -r file; do
    first=$(sed -n '1p' "$file")
    [ "$first" = '#!/bin/sh' ] || { printf '%s\n' "erro: shebang não POSIX em $file" >&2; exit 1; }
    sed "s/'[^']*'//g" "$file" > "$TMP_FILE"
    if grep -En '\[\[|\]\]|<<<|<\(|>\(|(^|[;[:space:]])source[[:space:]]|(^|[;[:space:]])declare[[:space:]]|(^|[;[:space:]])mapfile[[:space:]]|(^|[;[:space:]])coproc([;[:space:]]|$)' "$TMP_FILE" >/dev/null 2>&1; then
        printf '%s\n' "erro: possível bashism em $file" >&2
        exit 1
    fi
    sh -n "$file"
done

printf '%s\n' 'auditoria POSIX: OK'
