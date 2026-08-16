# ADR-E01-020 — Baseline de validação da fundação

**Status:** aceito — 2026-08-16

## Decisão

Uma revisão da fundação só é considerada compilada/testada quando os dois jobs canônicos — Arch Linux current e Debian Trixie — executarem com sucesso os wrappers da própria etapa.

Cada wrapper deve concluir build release, `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, `cargo doc` com warnings como erro e staging.

## Evidência inicial

Run `31974201820`, commit `42dc0a0713e4e21c772b3dac28b3edf47a0fab1a`: ambos os jobs concluíram com sucesso.

## Consequência

A CI não substitui testes de runtime de módulos futuros. Ela é a baseline da fundação e deverá evoluir quando novos contratos compartilhados forem introduzidos pelas etapas responsáveis.
