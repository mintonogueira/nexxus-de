# Status — Etapa 02

Data-base: 2026-08-16

Estado atual: **EM VALIDAÇÃO FINAL NA BRANCH DA ETAPA**.

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
- fontes normalizadas por `rustfmt` e `Cargo.lock` versionado pela CI.

## Evidência intermediária

- GitHub Actions run `31980740552`: sucesso em Arch Linux, Debian Trixie e geração de snapshot sobre a revisão anterior à normalização publicada.
- Commit de normalização/lock: `969c2f858f9b82353e861af9834db083df07ffa8`.

## Pendente para fechamento

- validar novamente a revisão já normalizada/versionada;
- registrar snapshot final e SHA-256;
- handoff final;
- integração/publicação na `main`.

Nenhuma etapa posterior foi iniciada nesta conversa.
