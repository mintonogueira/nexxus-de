# Nexxus — Etapa 05 — Workspace Manager

**Status:** `EM_IMPLEMENTACAO`

Implementação do módulo backend-neutral `nexxus-workspaces`, responsável por workspaces fixas/dinâmicas, workspace ativa, histórico/MRU determinístico, membership de janelas, placement inicial por aplicação e persistência da configuração.

## Fronteira arquitetural

- não existe vínculo rígido workspace ↔ monitor;
- a geometria e o foco interno das janelas continuam pertencendo ao `nexxus-wm`;
- regras de aplicação são avaliadas somente no primeiro placement e nunca impedem movimento manual posterior;
- configuração persistente não armazena processos/janelas em execução, preservando o escopo futuro de Session State;
- o crate não depende de X11, Wayland ou handles nativos de backend.

## Dependências de entrada

- Etapa 01: `nexxus-config` para persistência TOML versionada e atômica;
- Etapa 02: `nexxus-wm::WindowId` como identidade backend-neutral das janelas;
- Etapas 03–04: contratos de sessão/backend já validados e preservados, sem duplicação de suas responsabilidades.

## Validação

Os dois cenários de automação são Shell 100% POSIX e executam build release, rustfmt, Clippy com warnings como erro, testes, rustdoc, verificação de neutralidade de backend e staging. Como `nexxus-workspaces` é biblioteca interna nesta etapa, `NEXXUS_INSTALLABLE=0`: pacote nativo e instalação são N/A, sem fabricar pacote vazio.

Referência de entrada: `etapas/etapa-04-backend-x11/docs/HANDOFF_FINAL_ETAPA_04.md`.
