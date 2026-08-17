# ADR-001 — Modelo de tiling assistido e geometria

**Status:** ACEITO NA ETAPA 06  
**Data:** 2026-08-16

## Problema

O Nexxus exige tiling opcional, `Super+T`, snap por bordas, layouts distintos por workspace e suporte multi-monitor sem transformar a workspace em propriedade de um monitor nem aprisionar janelas no tiling.

## Decisão

1. `nexxus-tiling` permanece backend-neutral e consome somente contratos de `nexxus-wm` e `nexxus-workspaces`.
2. Slots são representados em fixed-point de 10.000 unidades, evitando float e garantindo cálculo determinístico.
3. Layout é propriedade lógica de `WorkspaceId`; o output entra somente no cálculo físico atual por `OutputArea`.
4. Assignments runtime usam `(workspace, output, window)` sem criar binding rígido `workspace -> output`.
5. O fallback interno enquanto Settings ainda não existe é um layout balanceado de duas colunas.
6. `tile-fit` calcula toda a geometria antes de alterar `WindowPlacement`, garantindo falha sem mutação parcial.
7. `nexxus-wm::set_placement(Tiled)` preserva a geometria floating; `untile` e início de move/resize manual voltam a `Floating` e restauram essa geometria.
8. Constraints máximas menores que o slot centralizam a janela dentro do slot. Constraints mínimas maiores que o slot geram erro explícito.
9. Snap direto usa metades/quartos. Topo central emite `ShowLayoutChoices`; o overlay definitivo fica fora desta etapa.
10. O contrato da ação é `nexxus.tiling.tile-fit` com binding aprovado `Super+T`. A captura global real pertence ao Shortcuts Core para evitar duplicação de responsabilidade.
11. Execução no X11 usa os `WmCommand` e `BackendCommandSink` existentes; nenhum tipo X11 entra no engine.

## Consequências

- uma mesma workspace pode distribuir janelas em vários outputs;
- o algoritmo é reutilizável pelo futuro backend Wayland;
- falhas de geometria/constraints não deixam janela parcialmente tiled;
- a UI futura recebe hooks sem ser antecipada;
- alterações de topologia de monitores podem apenas recalcular planos com novos `OutputArea`.
