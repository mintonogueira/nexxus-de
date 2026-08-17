#!/bin/sh
# Valida estrutura, segurança básica e escalabilidade dos assets da Etapa 08.
set -eu
SCRIPT_DIR=$(dirname "$0"); case "$SCRIPT_DIR" in -*) SCRIPT_DIR="./$SCRIPT_DIR" ;; esac
SCRIPT_DIR=$(CDPATH= cd "$SCRIPT_DIR" && pwd)
ROOT_DIR=$(CDPATH= cd "$SCRIPT_DIR/.." && pwd)
. "$ROOT_DIR/manifests/etapa-08.conf"
. "$ROOT_DIR/scripts/lib/common.sh"

require_command xmllint
require_command resvg
require_command find
require_command grep

icons_dir="$ROOT_DIR/assets/icons"
wallpapers_dir="$ROOT_DIR/assets/wallpapers"
tmp_dir="$ROOT_DIR/.build/asset-validation"
rm -rf "$tmp_dir"
mkdir -p "$tmp_dir"

icon_count=$(find "$icons_dir" -type f -name '*.svg' | wc -l | tr -d ' ')
wallpaper_count=$(find "$wallpapers_dir" -type f -name '*.svg' | wc -l | tr -d ' ')
[ "$icon_count" = "$NEXXUS_ICON_COUNT" ] || die "catálogo deve conter $NEXXUS_ICON_COUNT ícones; encontrado $icon_count"
[ "$wallpaper_count" = "$NEXXUS_WALLPAPER_COUNT" ] || die "pacote deve conter $NEXXUS_WALLPAPER_COUNT wallpapers; encontrado $wallpaper_count"

find "$ROOT_DIR/assets" -type f \( -name '*.ttf' -o -name '*.otf' -o -name '*.woff' -o -name '*.woff2' \) | grep . >/dev/null 2>&1 && die 'fonte binária vendorizada: Hack deve vir da distribuição' || :
[ -f "$ROOT_DIR/assets/manifest.toml" ] || die 'manifest.toml ausente'
[ -f "$ROOT_DIR/assets/LICENSES.md" ] || die 'registro de licenças ausente'
[ -f "$ROOT_DIR/assets/manifests/app-fallbacks.toml" ] || die 'fallback de aplicações ausente'
[ -f "$ROOT_DIR/assets/manifests/mime-fallbacks.toml" ] || die 'fallback MIME ausente'

# O namespace XML/SVG obrigatório contém uma URI HTTP, mas não é um recurso
# externo. Ele é removido apenas da cópia de auditoria antes de procurar URLs,
# hrefs, conteúdo ativo, filtros ou transparência decorativa.
validate_no_active_or_external_content() {
    file=$1
    audit_file="$tmp_dir/svg-audit.$$.txt"
    sed 's#xmlns="http://www.w3.org/2000/svg"##g' "$file" > "$audit_file"
    if grep -Eiq '<[[:space:]]*(script|image|filter)|https?://|xlink:href|[[:space:]]href=|[[:space:]]opacity=' "$audit_file"; then
        die "conteúdo SVG proibido ou externo em $file"
    fi
}

# Cada ícone é XML válido, simbólico 24x24 e renderizável em escalas pequenas e grandes.
find "$icons_dir" -type f -name '*.svg' | sort | while IFS= read -r file; do
    xmllint --noout "$file"
    grep -F 'viewBox="0 0 24 24"' "$file" >/dev/null || die "viewBox simbólico inválido em $file"
    grep -F '#FFFFFF' "$file" >/dev/null || die "token de recoloração ausente em $file"
    validate_no_active_or_external_content "$file"
    base=$(basename "$file" .svg)
    resvg --width 16 "$file" "$tmp_dir/${base}-16.png"
    resvg --width 64 "$file" "$tmp_dir/${base}-64.png"
    [ -s "$tmp_dir/${base}-16.png" ] && [ -s "$tmp_dir/${base}-64.png" ] || die "falha de rasterização em $file"
done

# Wallpapers permanecem opacos, locais e sem filtros/efeitos; renderização reduzida prova parse e escala.
find "$wallpapers_dir" -type f -name '*.svg' | sort | while IFS= read -r file; do
    xmllint --noout "$file"
    grep -F 'viewBox="0 0 1920 1080"' "$file" >/dev/null || die "viewBox de wallpaper inválido em $file"
    validate_no_active_or_external_content "$file"
    base=$(basename "$file" .svg)
    resvg --width 320 "$file" "$tmp_dir/${base}.png"
    [ -s "$tmp_dir/${base}.png" ] || die "falha de rasterização do wallpaper $file"
done

printf 'assets_validated icons=%s wallpapers=%s\n' "$icon_count" "$wallpaper_count" > "$ROOT_DIR/metrics/assets-validation.txt"
log_msg "[assets] $icon_count ícones e $wallpaper_count wallpapers validados"
