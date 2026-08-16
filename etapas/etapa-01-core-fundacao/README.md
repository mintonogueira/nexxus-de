# Nexxus — Etapa 01 — Core e Fundação Arquitetural

Estado: **VALIDADA TECNICAMENTE — fechamento material da entrega em preparação**.

Esta pasta contém exclusivamente a fundação arquitetural da Etapa 01. Não contém Window Manager, compositor X11/Wayland, tiling, workspaces, painel, UI, File Manager, Terminal, Settings, Keyring, Disks ou implementação funcional de etapas posteriores.

## Crates

- `nexxus-core`: identidade, dependências, registry, lifecycle, eventos e paths XDG/runtime.
- `nexxus-protocol`: protocolo IPC local versionado, framing limitado e Unix Domain Socket privado.
- `nexxus-config`: configuração TOML versionada com limite defensivo e escrita atômica.
- `nexxus-backend-api`: contratos abstratos para backends gráficos futuros; nenhuma implementação X11/Wayland.

## Validação

```sh
./scripts/check.sh
./scripts/build-install-arch.sh
./scripts/build-install-debian.sh
```

Os wrappers são `#!/bin/sh` 100% POSIX, validam a distribuição, executam build/testes como usuário normal e só elevam operações de gerenciamento de pacotes. Como a Etapa 01 ainda não produz um executável/serviço instalável, o manifesto declara `NEXXUS_INSTALLABLE=0`: staging é exercitado, mas pacote/instalação permanecem N/A para evitar artefatos vazios e estados falsos.

A revisão técnica `c714fe803fce32f59823d6d5ee7a217aa9d77d77` foi aprovada pela CI canônica, run `31974713059`, nos cenários Arch Linux current e Debian Trixie.

CI canônica: `.github/workflows/etapa-01-core.yml`.
