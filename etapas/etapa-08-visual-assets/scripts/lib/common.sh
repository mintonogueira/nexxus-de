#!/bin/sh
# Helpers POSIX compartilhados exclusivamente pelo fluxo da Etapa 08.

log_msg() {
    printf '%s\n' "$*"
}

die() {
    printf 'ERRO: %s\n' "$*" >&2
    exit 1
}

require_command() {
    command -v "$1" >/dev/null 2>&1 || die "comando obrigatório ausente: $1"
}

# O build e os testes devem ocorrer como usuário normal. Elevação é limitada
# à instalação de dependências e do pacote binário final.
require_unprivileged_user() {
    [ "$(id -u)" -ne 0 ] || die 'não execute o pipeline de build como root'
}

find_privilege_command() {
    if command -v sudo >/dev/null 2>&1; then
        PRIVILEGE_CMD='sudo'
    elif command -v doas >/dev/null 2>&1; then
        PRIVILEGE_CMD='doas'
    else
        die 'sudo ou doas é necessário somente para operações privilegiadas'
    fi
}

run_privileged() {
    "$PRIVILEGE_CMD" "$@"
}

# Remove somente diretórios de build internos previamente validados.
reset_build_dir() {
    target=$1
    case "$target" in
        "$ROOT_DIR"/.build/*) ;;
        *) die "recusa limpar caminho fora de .build: $target" ;;
    esac
    rm -rf "$target"
    mkdir -p "$target"
}

sha256_file() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$1" | awk '{print $1}'
    elif command -v shasum >/dev/null 2>&1; then
        shasum -a 256 "$1" | awk '{print $1}'
    else
        die 'sha256sum/shasum não encontrado'
    fi
}

# Garante que Rust/Cargo satisfazem a base mínima já congelada pelo Nexxus.
validate_rust_toolchain() {
    require_command cargo
    require_command rustc
    version=$(rustc --version | awk '{print $2}')
    major=$(printf '%s' "$version" | awk -F. '{print $1}')
    minor=$(printf '%s' "$version" | awk -F. '{print $2}')
    [ "$major" -gt 1 ] || { [ "$major" -eq 1 ] && [ "$minor" -ge 85 ]; } || die "Rust 1.85+ requerido; encontrado $version"
}

# Executa toda validação que deve preceder staging e empacotamento.
build_and_test_workspace() {
    sh "$ROOT_DIR/scripts/validate-assets.sh"
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace
    RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps
}

# Copia somente o payload runtime desta etapa para staging isolado.
prepare_staging() {
    STAGING_DIR="$BUILD_DIR/staging"
    rm -rf "$STAGING_DIR"
    dest="$STAGING_DIR$NEXXUS_ASSET_ROOT"
    mkdir -p "$dest"
    cp -R "$ROOT_DIR/assets/." "$dest/"
    [ -f "$dest/manifest.toml" ] || die 'manifest.toml não chegou ao staging'
    find "$STAGING_DIR" -type f \( -name '*.ttf' -o -name '*.otf' -o -name '*.woff' -o -name '*.woff2' \) | grep . >/dev/null 2>&1 && die 'fontes não devem ser vendorizadas no pacote' || :
}

verify_installed_payload() {
    [ -f "$NEXXUS_ASSET_ROOT/manifest.toml" ] || die 'manifesto não encontrado após instalação'
    [ -f "$NEXXUS_ASSET_ROOT/icons/actions/window-close.svg" ] || die 'ícone de smoke test ausente após instalação'
    [ -f "$NEXXUS_ASSET_ROOT/wallpapers/01-cyber-grid.svg" ] || die 'wallpaper de smoke test ausente após instalação'
    require_command fc-match
    fc-match -f '%{family}\n' Hack | head -n 1 | grep -i 'Hack' >/dev/null 2>&1 || die 'família Hack não resolvida pelo fontconfig após instalação'
}
