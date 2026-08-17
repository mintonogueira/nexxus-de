# NEXXUS — HANDOFF — ETAPA 13 — Desktop Shell

**Estado:** `VALIDADA_AGUARDANDO_PUBLICACAO`

## Identificação

- **Projeto:** Nexxus
- **Etapa:** 13 — Desktop Shell
- **Módulo:** `nexxus-desktop-shell`
- **Versão:** 0.1.0
- **Repositório:** `https://github.com/mintonogueira/nexxus-de`
- **Branch de implementação:** `etapa-13-desktop-shell-impl`
- **Destino canônico:** `main`

## Contratos consumidos

- Etapa 01 `nexxus-config`: persistência TOML atômica;
- Etapa 07 `nexxus-ui`: display list, renderer, geometria e tema;
- Etapa 08 `nexxus-assets`: wallpapers e ícones simbólicos/fallbacks;
- Etapa 10 `nexxus-shortcuts`: `ShellAction::DesktopMenu`;
- Etapa 12 `nexxus-xdg-application-index`: catálogo, categorias, launch templates e atualização dinâmica;
- backend X11 / EWMH / RandR: superfície inicial do desktop.

## Entregáveis

- crate `nexxus-desktop-shell`;
- persistência de wallpaper e launchers;
- pastas visíveis do XDG Desktop e criação segura de pasta;
- menu de contexto com Applications/categorias e ações semânticas;
- launch seguro por argv e ativação D-Bus;
- atualização dinâmica pelo serviço da Etapa 12;
- adapter X11 inicial com `_NET_WM_WINDOW_TYPE_DESKTOP` e RandR;
- testes unitários/integrados e smoke X11/Xvfb;
- scripts POSIX de preparação/build/validação Arch Linux e Debian;
- CI da etapa;
- `Cargo.lock`, snapshot `.tar.gz` e SHA-256 em `entrega/`.

## Validações executadas

- Arch Linux atual: APROVADO;
- Debian Trixie: APROVADO;
- `cargo fmt --check`: APROVADO;
- `cargo clippy --all-targets -- -D warnings`: APROVADO;
- `cargo test`: APROVADO;
- rustdoc com `-D warnings`: APROVADO;
- X11/Xvfb smoke test: APROVADO;
- delivery e versionamento de snapshot: APROVADOS.

## Decisões técnicas relevantes

- o índice de aplicações continua pertencendo exclusivamente à Etapa 12;
- ações de Terminal, File Manager e Desktop Settings permanecem semânticas, sem invasão dos respectivos módulos;
- o Desktop Shell é runtime integrável e permanece `NEXXUS_INSTALLABLE=0` nesta etapa;
- `zbus = 5.13.2` foi fixado localmente para preservar MSRV Rust 1.85;
- Cargo resolver v3 é utilizado no workspace da etapa.

## Pendência de fechamento

A implementação está tecnicamente validada. Falta apenas publicar o estado final na branch canônica `main` e confirmar a CI decorrente dessa publicação. Somente após isso a etapa será marcada `PUBLICADA`.
