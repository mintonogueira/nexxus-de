# Nexxus — Etapa 06 — Tiling Engine

**Status:** `VALIDADO / ENTREGUE / PUBLICADO`

Implementação do módulo backend-neutral `nexxus-tiling`, responsável por layouts por workspace, cálculo determinístico de slots/geometrias, ação pontual `tile-fit`, snap por bordas/topo e restauração da liberdade floating.

## Contratos preservados

- `nexxus-wm` continua proprietário do estado de janela, constraints e geometria floating;
- `nexxus-workspaces` continua proprietário da identidade/membership de workspaces;
- uma workspace não é vinculada rigidamente a monitor;
- a geometria física é resolvida por `OutputArea`, que representa somente o contexto de cálculo atual;
- o engine não contém tipos X11/Wayland e usa `WmCommand`/`BackendCommandSink` para chegar ao backend inicial X11;
- `Super+T` é exposto pelo contrato `nexxus.tiling.tile-fit`; a captura global definitiva permanece na Etapa 10 — Shortcuts Core;
- o topo central em snap produz hook para o futuro overlay, sem antecipar a UI definitiva.

## Escopo validado

- layout fallback balanceado em duas colunas;
- layouts explícitos distintos por workspace;
- slots normalizados em base fixa de 10.000 unidades;
- cálculo para áreas úteis e coordenadas multi-monitor, inclusive monitores à esquerda da origem;
- arredondamento determinístico de limites fracionários ao pixel físico mais próximo, sem criar gaps entre slots adjacentes;
- constraints min/max com erro seguro quando o mínimo não cabe;
- alocação determinística de slots por workspace + output;
- `tile_fit_active` para a semântica de `Super+T`;
- snap direto em metades/quartos nas bordas/cantos;
- hook `ShowLayoutChoices` no topo;
- `untile` e `release_for_manual_operation`;
- preservação/restauração da geometria floating pelo contrato existente do WM;
- eventos de tiling para integrações futuras;
- wrappers Shell 100% POSIX separados para Arch Linux e Debian;
- validação real em Arch Linux e Debian via GitHub Actions;
- snapshot versionado com SHA-256.

## Entrega

- implementação: PR #6;
- merge validado em `main`: `687efab9c3adf7a3a8d067c390fd374969625438`;
- workflow pós-merge: `31987843352` — Arch Linux e Debian aprovados;
- snapshot: `Nexxus_Etapa06_Tiling_Engine_0.1.0.tar.gz`;
- SHA-256: `f03bcd4d7b6de0de8d710ccccd821ee9761d7f1fa3fb9804efeae7143ca277d8`.

## Fora do escopo preservado

Overlay gráfico definitivo, Settings de tiling, Shortcuts Core completo, Window Chrome e backend Wayland específico.
