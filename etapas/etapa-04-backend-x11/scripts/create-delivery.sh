#!/bin/sh
# Gera o snapshot de entrega somente quando os dois pacotes nativos validados existem.
set -eu

SCRIPT_DIR=$(dirname "$0"); case "$SCRIPT_DIR" in -*) SCRIPT_DIR="./$SCRIPT_DIR" ;; esac
SCRIPT_DIR=$(CDPATH= cd "$SCRIPT_DIR" && pwd)
ROOT_DIR=$(CDPATH= cd "$SCRIPT_DIR/.." && pwd)
. "$ROOT_DIR/manifests/etapa-04.conf"

DELIVERY_BASENAME="Nexxus_Etapa04_Backend_X11_${NEXXUS_VERSION}"
DELIVERY_DIR="$ROOT_DIR/entrega"
BUILD_DIR="$ROOT_DIR/.build/delivery"
SNAPSHOT="$DELIVERY_DIR/${DELIVERY_BASENAME}.tar.gz"
CHECKSUM="$SNAPSHOT.sha256"

# O snapshot é uma entrega conjunta; não deve ser criado com apenas um dos
# cenários de empacotamento presentes, pois isso produziria evidência parcial.
set -- "$ROOT_DIR"/dist/arch/nexxus-backend-x11-*.pkg.tar.*
[ "$#" -eq 1 ] && [ -f "$1" ] || { printf '%s\n' 'erro: pacote Arch único não encontrado' >&2; exit 1; }
ARCH_PACKAGE=$1
set -- "$ROOT_DIR"/dist/debian/nexxus-backend-x11_*.deb
[ "$#" -eq 1 ] && [ -f "$1" ] || { printf '%s\n' 'erro: pacote Debian único não encontrado' >&2; exit 1; }
DEBIAN_PACKAGE=$1

case "$BUILD_DIR" in "$ROOT_DIR"/.build/*) ;; *) printf '%s\n' 'erro: caminho de staging inválido' >&2; exit 1 ;; esac
rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR/$DELIVERY_BASENAME" "$DELIVERY_DIR"

# Copia apenas material versionável/entregável da própria etapa. target/, .build/
# e o diretório entrega/ ficam de fora para impedir caches e recursão do arquivo.
for item in Cargo.toml Cargo.lock README.md crates docs manifests packaging scripts dist; do
    [ -e "$ROOT_DIR/$item" ] || { printf '%s\n' "erro: item obrigatório ausente: $item" >&2; exit 1; }
    cp -R "$ROOT_DIR/$item" "$BUILD_DIR/$DELIVERY_BASENAME/$item"
done

if command -v git >/dev/null 2>&1; then
    SOURCE_COMMIT=$(git -C "$ROOT_DIR" rev-parse HEAD 2>/dev/null || printf '%s' 'UNKNOWN')
else
    SOURCE_COMMIT='UNKNOWN'
fi
ARCH_SHA=$(sha256sum "$ARCH_PACKAGE" | awk '{print $1}')
DEBIAN_SHA=$(sha256sum "$DEBIAN_PACKAGE" | awk '{print $1}')

cat > "$BUILD_DIR/$DELIVERY_BASENAME/ENTREGA_MANIFESTO.txt" <<EOF
PROJETO=Nexxus
ETAPA=04 - Backend X11
VERSAO=$NEXXUS_VERSION
SOURCE_COMMIT=$SOURCE_COMMIT
PACOTE_ARCH=$(basename "$ARCH_PACKAGE")
SHA256_ARCH=$ARCH_SHA
PACOTE_DEBIAN=$(basename "$DEBIAN_PACKAGE")
SHA256_DEBIAN=$DEBIAN_SHA
BACKEND=x11
EWMH_ICCCM=validacao_integracao
COMPOSITOR=nao_requerido_nesta_etapa
EOF

rm -f "$SNAPSHOT" "$CHECKSUM"
(
    cd "$BUILD_DIR"
    tar -czf "$SNAPSHOT" "$DELIVERY_BASENAME"
)
sha256sum "$SNAPSHOT" > "$CHECKSUM"

# Validação mínima evita publicar um arquivo vazio/corrompido por erro de staging.
tar -tzf "$SNAPSHOT" >/dev/null
[ -s "$CHECKSUM" ] || { printf '%s\n' 'erro: checksum do snapshot não foi gerado' >&2; exit 1; }
printf '%s\n' "snapshot=$SNAPSHOT"
printf '%s\n' "sha256=$(awk '{print $1}' "$CHECKSUM")"
