# Nexxus — Etapa 04 — Backend X11

**Status:** `EM_VALIDACAO`

Implementação do primeiro backend gráfico concreto do Nexxus. Esta etapa conecta o `nexxus-wm` ao X11 sem incorporar Workspace Manager, Tiling Engine, UI final, Wayland ou Portals.

## Decisões técnicas vigentes

- binding X11: `x11rb 0.14.0`, conexão Rust pura, sem feature `allow-unsafe-code`;
- crate da etapa mantém `#![forbid(unsafe_code)]`;
- EWMH/ICCCM pertinentes são tratados no adapter X11;
- as janelas não são reparentadas nem decoradas nesta etapa, preservando CSD/SSD e a fronteira da futura Etapa 09 — Window Chrome;
- compositor X11 não é ativado: gerenciamento de janelas desta etapa não o exige tecnicamente e efeitos visuais continuam proibidos.

## Estado de engenharia

As fontes Rust já foram normalizadas por `rustfmt`; a implementação segue em validação funcional, Clippy, testes X11 reais sob Xvfb e empacotamento nativo Arch/Debian.

Branch de implementação: `etapa-04-backend-x11-impl`.
