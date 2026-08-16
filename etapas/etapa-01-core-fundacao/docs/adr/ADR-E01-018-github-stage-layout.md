# ADR-E01-018 — Organização GitHub por etapa

**Status:** aceito — 2026-08-16

## Decisão

Todo material específico desta conversa reside sob `etapas/etapa-01-core-fundacao/`. Arquivos tecnicamente compartilhados que exigem path especial, como `.github/workflows/etapa-01-core.yml`, podem permanecer fora dessa pasta desde que sua responsabilidade seja explicitamente atribuída à Etapa 01.

A branch canônica é `main` e os updates são exclusivamente fast-forward/não destrutivos.

## Razão

Implementa diretamente o Aditivo 11 preservando simultaneamente organização por etapa e requisitos técnicos do GitHub Actions.
