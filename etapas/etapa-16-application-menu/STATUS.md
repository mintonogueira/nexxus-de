# STATUS — Etapa 16 — Application Menu

- **Projeto:** Nexxus
- **Etapa:** 16 — Application Menu
- **Módulo:** `nexxus-app-menu`
- **Versão:** 0.1.0
- **Estado:** IMPLEMENTADO / CI PENDENTE
- **Branch:** `etapa-16-application-menu-impl-v2`
- **Base temporária:** `etapa-15-panel-core-impl`

## Implementado

- modelo de estado do menu;
- categorias, favoritos, recentes e todos os aplicativos;
- busca instantânea pelo XDG Application Index;
- modos List/Grid e tamanhos de ícone;
- shell-free launch command;
- adaptador `PanelPlugin` compatível com API 1.0;
- testes de integração;
- scripts POSIX Arch/Debian;
- workflow CI Arch Linux/Debian.

## Bloqueios externos

A Etapa 15 permanece fora de `main`; sua PR #18 está aberta e o último CI conhecido falhou em `cargo fmt --check`. Esta etapa não altera código da Etapa 15 e permanece empilhada sobre a branch dela até que o Panel Core seja validado/integrado.
