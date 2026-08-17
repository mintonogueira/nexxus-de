# Status — Etapa 02

Data-base: 2026-08-16

Estado atual: **VALIDADA — ENTREGA 0.1.0 PRONTA PARA `main`**.

## Implementado

- `nexxus-wm` backend-neutral;
- modelo de janelas, geometria e constraints;
- lifecycle lógico;
- foco/MRU determinísticos;
- maximize/restore e fullscreen/restore;
- placement floating/tiled sem implementar algoritmo de tiling;
- contratos de eventos/comandos para futuros backends;
- testes unitários e de integração;
- wrappers Shell POSIX separados para Arch Linux e Debian;
- workflow GitHub Actions da Etapa 02;
- `Cargo.lock` gerado pelo Cargo e versionado;
- scripts operacionais marcados executáveis;
- snapshot e SHA-256 versionados.

## Evidência final da branch

- Commit técnico validado: `9cbc6d9fde1f40f0a9b49b525ced8e798f156d95`.
- GitHub Actions run final: `31980820527` — success.
- Arch Linux current: success.
- Debian Trixie: success.
- `snapshot-entrega`: success.
- Build release, rustfmt, Clippy `-D warnings`, testes, rustdoc, auditoria POSIX e neutralidade de backend: aprovados.

## Entrega compactada

- Arquivo: `entrega/Nexxus_Etapa02_Window_Manager_Core_0.1.0.tar.gz`.
- SHA-256: `26c39a1f87e2d0bc5fe6f1bfa9dd77437b354fe7f62eb59e70224889644ed9e1`.
- Commit que adiciona snapshot e modos executáveis: `ab51cd12c5663fd29d9131b8f62dc7338f627d41`.

## Distribuição

- REVISADO: SIM.
- COMPILADO: SIM.
- TESTADO: SIM.
- EMPACOTADO NATIVO: N/A — biblioteca interna sem payload runtime.
- INSTALADO: N/A.
- VALIDADO: SIM.
- ENTREGA_COMPACTADA: VALIDADA / VERSIONADA.

Nenhuma etapa posterior foi iniciada nesta conversa. O próximo passo desta etapa é exclusivamente integrar a entrega validada à `main` e registrar o handoff final.
