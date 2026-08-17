# ADR-001 — Binding X11 e política de compositor da Etapa 04

**Status:** APROVADO_TECNICAMENTE  
**Data:** 2026-08-16

## Contexto

A Etapa 04 exige X11/XCB, EWMH/ICCCM, baixo footprint, Rust-first, FFI/unsafe restrito e compositor somente quando tecnicamente necessário.

## Decisão

1. Adotar `x11rb 0.14.0` com conexão Rust pura e `default-features = false`.
2. Não habilitar `allow-unsafe-code`/XCB FFI; o crate Nexxus mantém `#![forbid(unsafe_code)]`.
3. Não iniciar compositor X11 nesta etapa. O gerenciamento EWMH/ICCCM, foco, geometria e lifecycle funciona diretamente sobre o X server sem Composite/Damage.
4. Não reparentar nem desenhar decoração. CSD de aplicações permanece intacto e SSD/Nexxus Window Chrome pertence à Etapa 09.

## Motivo

A solução reduz dependências nativas, fronteiras unsafe, memória e complexidade. Atende o resultado funcional sem introduzir efeitos proibidos ou antecipar módulos posteriores.

## Impacto

- multi-monitor/hotplug permanecem hooks futuros e não são anunciados como capability nesta etapa;
- se uma necessidade real de composição surgir futuramente, ela deverá ser justificada e implementada sem efeitos estéticos, respeitando a governança aditiva.

## Fontes primárias

- X.Org X11/ICCCM;
- freedesktop.org EWMH;
- XCB;
- upstream `psychon/x11rb` v0.14.0.
