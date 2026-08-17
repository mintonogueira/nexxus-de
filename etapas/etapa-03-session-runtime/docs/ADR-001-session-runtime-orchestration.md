# ADR-E03-001 — Orquestração do Session Runtime

**Status:** APROVADO INTERNAMENTE NA ETAPA 03  
**Data:** 2026-08-16

## Contexto

A Etapa 03 precisa coordenar bootstrap e shutdown sem incorporar X11, Wayland ou lógica do Window Manager.

## Decisão

1. Reutilizar `ModuleRegistry`, `LifecycleManager`, `NexxusPaths` e `UnixEndpoint` da Etapa 01.
2. Consumir `nexxus-wm` por um adapter de lifecycle que apenas cria/descarta `WindowManager`; nenhuma regra de janelas é reimplementada.
3. Exigir backend explícito por CLI/configuração. Ausência é erro; seleção explícita indisponível é erro.
4. Receber o backend concreto por `BackendModule`, preservando a fronteira da Etapa 03 e permitindo que etapas de backend forneçam a implementação sem colocar protocolo gráfico no runtime.
5. Reservar `session.sock` no runtime privado para controle/diagnóstico mínimo.
6. Startup usa a ordem resolvida pela fundação; shutdown usa exatamente a ordem inversa e continua cleanup após falhas individuais.

## Consequências

- A Etapa 03 é testável com backend sintético sem antecipar X11/Wayland.
- O binário base não promete iniciar sessão gráfica antes da existência de backend concreto.
- A integração futura precisa fornecer um `NexxusModule` com id canônico do backend e capability `graphics.backend`.
- Nenhuma ABI dinâmica ou `unsafe` é introduzida.
