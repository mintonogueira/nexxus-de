# Nexxus — Etapa 01 — Core e Fundação Arquitetural

Estado: **EM DESENVOLVIMENTO**.

Esta pasta pertence exclusivamente à Etapa 01. Não contém implementação de Window Manager, compositor X11/Wayland, tiling, painel, File Manager, Terminal, Settings, Keyring, Disks ou módulos funcionais de etapas posteriores.

## Fundação implementada

- `nexxus-core`: identidade, dependências explícitas, capabilities, registry, lifecycle com preflight/rollback, Event Bus e paths XDG/runtime privados.
- `nexxus-protocol`: protocolo IPC privado versionado, framing limitado e Unix Domain Socket com endpoint privado.
- `nexxus-config`: TOML versionado, limite defensivo e gravação atômica.
- `nexxus-backend-api`: contrato mínimo abstrato para backends gráficos futuros; sem implementação X11/Wayland.

## Build e empacotamento

Entradas oficiais desta etapa:

```sh
./scripts/build-install-arch.sh
./scripts/build-install-debian.sh
```

Os wrappers são `#!/bin/sh` POSIX e autoprovisionam dependências ausentes da própria distribuição. Build/testes são recusados como root.

A Etapa 01 ainda não produz um executável/serviço instalável. O manifesto usa `NEXXUS_INSTALLABLE=0`; portanto os wrappers validam build, testes e staging, mas não fabricam pacote vazio nem declaram `EMPACOTADO`/`INSTALADO`.

## Validação direta do workspace

```sh
./scripts/check-posix.sh
./scripts/check.sh
```

`check.sh` executa `rustfmt`, `clippy -D warnings`, testes e `rustdoc -D warnings`.

## Toolchain

- Rust Edition 2024.
- MSRV: Rust 1.85, compatível com a toolchain empacotada no Debian 13 e suficiente para Edition 2024.
- Licença do código: `GPL-3.0-only`, coerente com o `LICENSE` da raiz do repositório canônico.

## Repositório

A publicação desta etapa pertence a:

`etapas/etapa-01-core-fundacao/`

no repositório canônico `https://github.com/mintonogueira/nexxus-de`, branch `main`.
