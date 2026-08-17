# Status — Etapa 02

Data-base: 2026-08-16

Estado atual: **VALIDADA E ENTREGUE — versão 0.1.0**.

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
- snapshot e SHA-256 versionados;
- handoff final.

## Evidência final

- Commit técnico validado na branch: `539f09fbbdba2382d41abf76c1f2db8739b9be71`.
- GitHub Actions da branch: run `31981061198` — success.
- Pull Request: `#1` — merged.
- Commit de merge na `main`: `096271cfdfb3f50b624e4560bb86c7c7b731dcc1`.
- GitHub Actions pós-merge na `main`: run `31981149587` — success.
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
- ENTREGA_COMPACTADA: VALIDADA / PUBLICADA.
- STATUS_GITHUB: PUBLICADO NA `main`.

## Encerramento

Nenhuma pendência funcional conhecida permanece dentro do escopo da Etapa 02. Nenhuma etapa posterior foi iniciada nesta conversa.

Próxima etapa recomendada: **Etapa 03 — Session Runtime**, obrigatoriamente em nova conversa.
