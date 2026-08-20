# STATUS — Etapa 16 — Application Menu

- **Projeto:** Nexxus
- **Etapa:** 16 — Application Menu
- **Módulo:** `nexxus-app-menu`
- **Versão:** 0.1.0
- **Estado:** VALIDADO_NO_ESCOPO / BLOQUEADO_PARA_INTEGRAÇÃO
- **Branch:** `etapa-16-application-menu-impl-v2`
- **Pull request:** #19
- **Base temporária:** `etapa-15-panel-core-impl`
- **Commit validado:** `ba053f32e496213a6c5218466c4f1b0d3ca75f0e`
- **GitHub Actions:** run `32329631563` — sucesso em Arch Linux e Debian Trixie

## Implementado e validado

- modelo de estado do menu;
- categorias, favoritos, recentes e todos os aplicativos;
- busca instantânea pelo XDG Application Index;
- modos List/Grid e tamanhos de ícone;
- preservação dos ícones entregues pela Etapa 12;
- shell-free launch command;
- adaptador `PanelPlugin` compatível com API 1.0;
- testes de integração;
- scripts POSIX Arch/Debian;
- CI com format, testes, Clippy `-D warnings` e rustdoc.

## Bloqueio externo

A Etapa 15 permanece fora de `main`; a PR #18 está aberta e o último CI conhecido da Etapa 15 falhou em `cargo fmt --check`. A Etapa 16 não altera o código da Etapa 15 e permanece empilhada sobre a branch dela até o Panel Core ser validado e integrado.
