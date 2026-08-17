# STATUS — Etapa 05 — Workspace Manager

- **Projeto:** Nexxus
- **Etapa:** 05 — Workspace Manager
- **Módulo:** `nexxus-workspaces`
- **Versão:** 0.1.0
- **Status:** VALIDADO / PUBLICADO / ENTREGUE
- **Repositório:** `https://github.com/mintonogueira/nexxus-de`
- **Branch canônica:** `main`
- **Branch de implementação:** `etapa-05-workspace-manager-impl`
- **PR de implementação:** `#4`
- **Commit main validado:** `cd2e0d32445076f54701f9b724a3532eda3fd01f`

## Implementado

- modelo backend-neutral de workspace;
- workspaces fixas e dinâmicas;
- workspace ativa e MRU determinístico;
- membership e movimentação manual de janelas;
- placement inicial por `application_id` sem aprisionamento posterior;
- remoção segura sem perda de janelas;
- política configurável de retenção de workspaces dinâmicas vazias;
- persistência TOML versionada via `nexxus-config`;
- eventos create/remove/rename/activate/move-window/forget-window;
- base para Alt+Tab filtrado pela workspace atual e Super+Tab por MRU de workspaces;
- testes de lifecycle, MRU, placement, remoção e persistência;
- wrappers POSIX separados para Arch Linux e Debian;
- auditorias de POSIX e neutralidade de backend;
- snapshot `.tar.gz` e SHA-256 versionados.

## Validação

- workflow branch `31985950552`: Arch Linux **SUCCESS**, Debian **SUCCESS**, delivery **SUCCESS**;
- workflow main `31986088565`: Arch Linux **SUCCESS**, Debian **SUCCESS**, delivery **SKIPPED** conforme condição intencional do workflow;
- build release, rustfmt, Clippy `-D warnings`, testes, rustdoc `-D warnings`, neutralidade de backend e staging: **ATENDIDOS**;
- pacote nativo/instalação: **N/A**, pois o módulo é biblioteca interna (`NEXXUS_INSTALLABLE=0`) e não possui payload runtime independente nesta etapa.

## Entrega

- `entrega/Nexxus_Etapa05_Workspace_Manager_0.1.0.tar.gz`;
- SHA-256: `9923ed76531f613c02ff676665e863575924e6306ac51e140b3d962cc255bb4e`.

## Pendências da etapa

Nenhuma pendência bloqueante pertence à Etapa 05.

## Fora do escopo preservado

Tiling Engine, Workspace Bar, Settings gráfico de workspaces, Session State completo e backend Wayland/XWayland.

## Próxima etapa

**Etapa 06 — Tiling Engine**. Deve ser iniciada exclusivamente em nova conversa: `NEXXUS - Etapa 06 - Tiling Engine`.
