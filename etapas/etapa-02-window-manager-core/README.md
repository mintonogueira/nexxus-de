# Nexxus — Etapa 02 — Window Manager Core

Estado: **VALIDADA — versão 0.1.0, pronta para integração na `main`**.

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

## Validação final

GitHub Actions run `31980820527`:

- Arch Linux current: success;
- Debian Trixie: success;
- snapshot-entrega: success;
- build release, `rustfmt`, Clippy `-D warnings`, testes e rustdoc: aprovados;
- auditoria Shell POSIX: aprovada;
- auditoria de neutralidade X11/Wayland: aprovada.

## Entrega

- `entrega/Nexxus_Etapa02_Window_Manager_Core_0.1.0.tar.gz`
- SHA-256: `26c39a1f87e2d0bc5fe6f1bfa9dd77437b354fe7f62eb59e70224889644ed9e1`
- handoff: `docs/HANDOFF_FINAL_ETAPA_02.md`

A etapa é biblioteca interna e declara `NEXXUS_INSTALLABLE=0`; pacote nativo e instalação permanecem N/A porque não existe payload runtime instalável nesta etapa.
