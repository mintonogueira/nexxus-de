# NEXXUS — ETAPA 13 — Desktop Shell

**Versão:** 0.1.0  
**Status:** VALIDADA / AGUARDANDO PUBLICAÇÃO EM `main`

Implementação da superfície de desktop do Nexxus para o backend X11 inicial, consumindo os contratos já validados de `nexxus-ui`, Visual Assets, Shortcuts Core e XDG Application Index.

## Escopo implementado

- wallpaper persistente com fallback distribuído pelo Nexxus;
- launchers persistentes vinculados a Desktop File IDs validados pelo índice comum;
- ícones de pastas do Desktop e criação segura de nova pasta;
- menu de contexto por botão direito e pelo comando semântico `nexxus.shell.desktop-menu` (`Ctrl+Esc`);
- navegação de Applications por categorias do XDG Application Index;
- ações Terminal, File Manager, Create Folder, Create Launcher e Desktop Settings;
- launch de aplicações sem shell, com `Exec` seguro e suporte a `DBusActivatable`;
- atualização dinâmica quando a Etapa 12 publica nova geração do índice;
- uma única superfície X11 para o desktop, com posicionamento de menu por monitor via RandR;
- UI exclusiva em Rust por `nexxus-ui`, sem GTK/Qt.

## Validação

A implementação foi validada em Arch Linux atual e Debian Trixie por CI da própria etapa, incluindo `cargo fmt --check`, `cargo clippy -D warnings`, testes Rust, rustdoc com warnings negados e smoke test X11 em Xvfb. O delivery gera e versiona `Cargo.lock`, snapshot `.tar.gz` e arquivo SHA-256 correspondente.

## Fronteiras preservadas

O módulo não implementa o File Manager completo, a Central de Settings, o Terminal, o backend Wayland final nem um novo índice de aplicações. Ações pertencentes a módulos futuros são devolvidas semanticamente ao coordenador da sessão.

Consulte `docs/ADR-001-desktop-shell-contract.md` para decisões internas e `HANDOFF.md` para o fechamento da etapa.
