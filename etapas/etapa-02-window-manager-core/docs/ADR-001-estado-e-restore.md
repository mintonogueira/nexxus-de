# ADR-001 — Modelo de estado e restauração do Window Manager Core

Status: APROVADO INTERNAMENTE NA ETAPA 02  
Data: 2026-08-16

## Problema

O núcleo precisa representar floating/tiled, maximized/fullscreen e restauração sem incorporar o Tiling Engine nem detalhes de X11/Wayland.

## Decisão

Separar dois eixos de estado:

- `WindowPlacement`: `Floating` ou `Tiled`;
- `PresentationState`: `Normal`, `Maximized` ou `Fullscreen`.

Transições de apresentação guardam `RestoreSnapshot` em pilha curta e determinística, contendo geometria, placement e presentation anteriores. A geometria floating também é preservada separadamente para permitir retorno de `Tiled` para `Floating` sem depender do futuro algoritmo de tiling.

## Razão

A separação evita um enum combinatório de estados, preserva restauração aninhada como maximized → fullscreen → maximized → normal e mantém o escopo do Tiling Engine fora da Etapa 02.

## Consequências

- o core sabe **qual estado lógico** uma janela possui;
- o core não calcula slots/layouts de tiling;
- backends futuros recebem comandos abstratos e devolvem geometria/eventos normalizados;
- nenhum handle nativo X11/Wayland entra no modelo lógico.

## Alternativas descartadas

- Um único enum com todas as combinações: aumenta estados inválidos e acoplamento.
- Delegar toda restauração ao backend: duplicaria inteligência entre X11 e Wayland e quebraria o objetivo desta etapa.
