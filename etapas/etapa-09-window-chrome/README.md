# Nexxus — Etapa 09 — Window Chrome

Implementação inicial da decoração própria de janelas do Nexxus para X11 quando SSD é aplicável.

## Escopo desta etapa

- titlebar e bordas próprias;
- botões `tile-fit`, `maximize/restore` e `close`;
- move por titlebar;
- resize por bordas e cantos;
- estados visuais imediatos, sem animação;
- CSD preservada sem dupla decoração;
- escala e hit targets de mouse;
- integração com WM Core, Backend X11, Tiling Engine, Nexxus UI Core e Visual Assets.

Não fazem parte desta etapa: Settings completo de janelas, decoração Wayland final e minimizar globalmente.

## Arquitetura

O crate `nexxus-window-chrome` separa cinco responsabilidades:

- `policy`: decisão CSD/SSD e tipos X11 que não devem receber decoração;
- `geometry`: métricas, hit-testing e cálculo de resize;
- `render`: composição opaca da titlebar usando `nexxus-ui` e `nexxus-assets`;
- `integration`: adaptação dos contratos já entregues por WM/Tiling/X11;
- `x11`: superfícies SSD X11, eventos de mouse e `_NET_FRAME_EXTENTS`.

O adapter X11 não reparenta janelas de aplicação. O Backend X11 da Etapa 04 continua sendo o proprietário da operação física das janelas cliente; o Window Chrome cria somente superfícies próprias de decoração e delega move/resize/maximize/restore/close ao controller existente.

## Validação

```sh
sh ./scripts/check-posix.sh
sh ./scripts/build-install-arch.sh
# ou
sh ./scripts/build-install-debian.sh
```

Os wrappers Arch e Debian são `/bin/sh` POSIX e autoprovisionam as dependências do cenário. O módulo não possui payload executável independente nesta etapa; portanto pacote nativo/instalação são `N/A`, sem fabricação de pacote vazio.
