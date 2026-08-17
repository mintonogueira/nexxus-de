# Nexxus — Etapa 05 — Workspace Manager

**Status:** `VALIDADO / PUBLICADO / ENTREGUE`

Implementação do módulo backend-neutral `nexxus-workspaces`, responsável por workspaces fixas/dinâmicas, workspace ativa, histórico/MRU determinístico, membership de janelas, placement inicial por aplicação e persistência da configuração.

## Fronteira arquitetural

- não existe vínculo rígido workspace ↔ monitor;
- a geometria e o foco interno das janelas continuam pertencendo ao `nexxus-wm`;
- regras de aplicação são avaliadas somente no primeiro placement e nunca impedem movimento manual posterior;
- configuração persistente não armazena processos/janelas em execução, preservando o escopo futuro de Session State;
- o crate não depende de X11, Wayland ou handles nativos de backend;
- eventos backend-neutral constituem o contrato consumível por barra de workspaces, atalhos e adaptadores gráficos posteriores.

## Dependências de entrada

- Etapa 01: `nexxus-config` para persistência TOML versionada e atômica;
- Etapa 02: `nexxus-wm::WindowId` como identidade backend-neutral das janelas;
- Etapas 03–04: contratos de sessão/backend validados e preservados, sem duplicação de suas responsabilidades.

## Validação e publicação

- workflow da branch de implementação `31985950552`: Arch Linux **SUCCESS**, Debian **SUCCESS**, delivery **SUCCESS**;
- PR de implementação: `#4`;
- merge validado na `main`: `cd2e0d32445076f54701f9b724a3532eda3fd01f`;
- workflow de revalidação da `main` `31986088565`: Arch Linux **SUCCESS**, Debian **SUCCESS**; delivery **SKIPPED** por desenho, pois os artefatos já foram produzidos na branch de implementação;
- snapshot: `entrega/Nexxus_Etapa05_Workspace_Manager_0.1.0.tar.gz`;
- SHA-256: `9923ed76531f613c02ff676665e863575924e6306ac51e140b3d962cc255bb4e`.

Os dois cenários de automação são Shell 100% POSIX e executam build release, rustfmt, Clippy com warnings como erro, testes, rustdoc, verificação de neutralidade de backend e staging. Como `nexxus-workspaces` é biblioteca interna nesta etapa, `NEXXUS_INSTALLABLE=0`: pacote nativo e instalação são N/A, sem fabricar pacote vazio.

Handoff final: `docs/HANDOFF_FINAL_ETAPA_05.md`.
