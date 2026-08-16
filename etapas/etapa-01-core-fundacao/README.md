# Nexxus — Etapa 01 — Core e Fundação Arquitetural

Estado: **EM DESENVOLVIMENTO — fundação compilada e testada em Arch Linux current e Debian Trixie**.

Esta pasta contém exclusivamente a fundação arquitetural da Etapa 01. Não contém Window Manager, compositor X11/Wayland, tiling, workspaces, painel, UI, File Manager, Terminal, Settings, Keyring, Disks ou implementação funcional de etapas posteriores.

## Crates

- `nexxus-core`: identidade, dependências, registry, lifecycle, eventos e paths XDG/runtime.
- `nexxus-protocol`: protocolo IPC local versionado, framing limitado e Unix Domain Socket privado.
- `nexxus-config`: configuração TOML versionada com limite defensivo e escrita atômica.
- `nexxus-backend-api`: contratos abstratos para backends gráficos futuros; nenhuma implementação X11/Wayland.

## Decisões técnicas vigentes

- Rust Edition 2024; MSRV 1.85 para a fundação atual.
- `#![forbid(unsafe_code)]` nos quatro crates.
- Core sem runtime assíncrono obrigatório.
- IPC local por Unix Domain Socket, framing de tamanho explícito e limite de 1 MiB.
- JSON/Serde no wire interno inicial; protocolo `major.minor` versionado.
- TOML/Serde para configuração; arquivo temporário + `fsync` + `rename` no mesmo filesystem.
- Erros tipados com `thiserror` e instrumentação estrutural com `tracing`.
- Licença herdada do repositório canônico: `GPL-3.0-only`.

## Validação

Validação comum Rust:

```sh
./scripts/check.sh
```

Fluxos completos da infraestrutura da etapa:

```sh
./scripts/build-install-arch.sh
./scripts/build-install-debian.sh
```

Os wrappers são `#!/bin/sh` 100% POSIX, validam a distribuição, executam build/testes como usuário normal e só elevam operações de gerenciamento de pacotes. Como a Etapa 01 ainda não produz um executável/serviço instalável, o manifesto declara `NEXXUS_INSTALLABLE=0`: staging é exercitado, mas pacote/instalação permanecem N/A para evitar artefatos vazios e estados falsos.

CI canônica: `.github/workflows/etapa-01-core.yml`.
