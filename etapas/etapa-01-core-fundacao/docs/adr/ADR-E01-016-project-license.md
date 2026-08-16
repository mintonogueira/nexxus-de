# ADR-E01-016 — Licença declarada no workspace

**Status:** aceito — 2026-08-16

## Contexto

A documentação funcional não congelava uma licença, mas o repositório GitHub canônico definido pelo Aditivo 11 já contém `LICENSE` com GNU General Public License v3. A fundação deve manter metadados Cargo coerentes com a licença do repositório em que o projeto é oficialmente versionado.

## Decisão

Os packages desta Etapa 01 declaram `GPL-3.0-only` via `workspace.package.license`.

## Consequência

A decisão apenas harmoniza metadados do código com a licença já presente no repositório canônico; não introduz licença concorrente nem duplica o texto da licença dentro de cada crate.
