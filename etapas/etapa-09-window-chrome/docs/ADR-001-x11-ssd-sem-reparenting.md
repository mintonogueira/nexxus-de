# ADR-001 — Chrome SSD X11 sem reparenting da janela cliente

**Status:** APROVADO COMO DECISÃO TÉCNICA INTERNA DA ETAPA 09  
**Data:** 2026-08-17  
**Escopo:** Window Chrome X11 inicial

## Contexto

A Etapa 04 já entrega um Backend X11 que opera diretamente as janelas cliente e mantém a tradução entre eventos X11 e o WM Core. A Etapa 09 precisa adicionar chrome SSD sem reimplementar esse backend nem alterar seu contrato.

## Decisão

O Window Chrome usa uma conexão X11 própria para suas superfícies de decoração e não reparenta a janela cliente.

- a janela cliente continua sob responsabilidade do Backend X11 existente;
- titlebar/bordas/grabs são superfícies `override_redirect` pertencentes ao Window Chrome;
- move, resize, maximize, restore e close são delegados ao `X11Controller` da Etapa 04;
- `_NET_FRAME_EXTENTS` informa os extents da decoração SSD;
- a decisão CSD/SSD é conservadora: sinais de CSD e tipos especiais impedem a criação de chrome Nexxus;
- fullscreen remove a decoração enquanto o estado estiver ativo;
- o tile-fit e a liberação antes de move/resize manual são delegados ao Tiling Engine da Etapa 06.

## Motivação

Esta solução preserva o isolamento das etapas, evita duplicar o Window Manager/Backend X11, mantém o core agnóstico de backend e permite validar a experiência SSD inicial exigida pela Etapa 09.

## Consequências

- o Window Chrome precisa reconciliar geometria/estado com snapshots do controller;
- stacking e lifecycle das superfícies de decoração permanecem responsabilidade desta etapa;
- o desenho Wayland final não é derivado desta decisão e permanece fora do escopo;
- qualquer evolução que exija alterar contratos públicos das etapas 02/04/06 deverá retornar à etapa proprietária/coordenação.

## Segurança e qualidade

O crate mantém `#![forbid(unsafe_code)]`. Não usa GTK/Qt, não introduz animação/transparência decorativa e não implementa minimizar globalmente.
