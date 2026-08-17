# NEXXUS — HANDOFF — ETAPA 12 — XDG Application Index

**Estado:** EM IMPLEMENTAÇÃO — este arquivo será fechado somente após CI, snapshot, SHA-256 e publicação validados.

## Contrato produzido

O crate `nexxus-xdg-application-index` expõe:

- `ApplicationIndexConfig` e raízes XDG/Flatpak;
- `scan()` para geração síncrona de `IndexSnapshot`;
- `ApplicationIndexService` para atualização dinâmica;
- `ApplicationRecord`, `DesktopId`, `MainCategory` e `IconReference`;
- `ExecTemplate`/`LaunchCommand` como contrato shell-free de execução futura;
- `ApplicationIndexEvent::Changed(IndexDelta)` para consumidores reativos.

## Limites preservados

Menu visual, Desktop Shell, Desktop Context Menu e Finder visual permanecem fora da Etapa 12.

## Validação/publicação

Pendente até a execução real do workflow da Etapa 12. Nenhum resultado de CI, hash de snapshot ou commit validado é declarado antecipadamente.
