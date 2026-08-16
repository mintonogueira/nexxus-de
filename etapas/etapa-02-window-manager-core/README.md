# Nexxus — Etapa 02 — Window Manager Core

Estado: **EM IMPLEMENTAÇÃO / VALIDAÇÃO**.

Esta etapa implementa exclusivamente o núcleo lógico e agnóstico de backend do gerenciamento de janelas. Não contém Backend X11, Backend Wayland/XWayland, compositor, Workspace Manager, Tiling Engine, window chrome, painel ou UI.

## Entrega técnica

- crate `nexxus-wm`;
- identificadores internos de janela;
- geometria e restrições de tamanho;
- lifecycle lógico de criação, map/unmap e destruição;
- foco e ordem MRU determinísticos;
- estados `Floating`, `Tiled`, `Maximized` e `Fullscreen`;
- restauração de geometria/estado anterior;
- metadados normalizados de aplicação;
- eventos `WmEvent` e comandos `WmCommand` independentes de protocolo gráfico;
- `BackendCommandSink` como fronteira abstrata de saída para backends futuros.

## Dependências da fundação

A Etapa 02 consome, sem reconstruir:

- `nexxus-core`;
- `nexxus-backend-api`.

`nexxus-protocol` e `nexxus-config` não são dependências desta revisão porque o contrato atual não exige IPC nem persistência própria.

## Validação

```sh
sh scripts/check-posix.sh
sh scripts/check.sh
sh scripts/check-neutrality.sh
sh scripts/build-install-arch.sh      # em Arch Linux
sh scripts/build-install-debian.sh    # em Debian
```

A etapa é biblioteca interna e declara `NEXXUS_INSTALLABLE=0`; pacote nativo e instalação permanecem N/A enquanto não existir payload runtime.
