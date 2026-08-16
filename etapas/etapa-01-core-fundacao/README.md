# Nexxus — Etapa 01 — Core e Fundação Arquitetural

Estado: **VALIDADA E ENTREGUE — versão 0.1.0**.

Esta pasta contém exclusivamente a fundação arquitetural da Etapa 01. Não contém implementação funcional de etapas posteriores.

## Crates

- `nexxus-core`: identidade, dependências, registry, lifecycle, eventos e paths XDG/runtime.
- `nexxus-protocol`: IPC local versionado, framing limitado e Unix Domain Socket privado.
- `nexxus-config`: configuração TOML versionada com limite defensivo e escrita atômica.
- `nexxus-backend-api`: contratos abstratos para backends gráficos futuros.

## Validação final

GitHub Actions run `31975039874`:

- Arch Linux current: success
- Debian Trixie: success
- snapshot-entrega: success

Build Release, rustfmt, Clippy `-D warnings`, testes, rustdoc, staging e auditoria POSIX foram aprovados.

## Entrega

- `entrega/Nexxus_Etapa01_Core_0.1.0.tar.gz`
- SHA-256: `06928c110a20b65a0019e3dbe856b35ada3310f772d3e93c4b2874d54b361b7d`
- Handoff: `docs/HANDOFF_FINAL_ETAPA_01.md`

Como não existe payload de runtime instalável nesta etapa, pacote nativo e instalação permanecem N/A; nenhum pacote vazio é criado.
