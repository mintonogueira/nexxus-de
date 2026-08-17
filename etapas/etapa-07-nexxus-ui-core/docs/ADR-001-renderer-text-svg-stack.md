# ADR-001 — Renderer, texto e SVG do Nexxus UI Core

**Status:** ACEITA NA ETAPA 07  
**Natureza:** decisão interna de implementação; não altera requisito normativo.

## Contexto

A Etapa 07 precisa de texto Unicode com shaping/fallback, SVG e um primeiro target compatível com X11, sem acoplar a API pública a X11/Wayland e sem introduzir GTK/Qt. O workspace Nexxus mantém `rust-version = 1.85`.

## Decisão

1. `nexxus-ui` usa uma `DisplayList` backend-neutral e um `SoftwareRenderer` que entrega frame RGBA8. Apresentação X11/Wayland permanece responsabilidade dos respectivos adapters/backends.
2. Texto usa `cosmic-text = 0.16.0`, que combina descoberta/fallback de fontes, shaping avançado, bidirecionalidade e rasterização em Rust. Esta versão declara MSRV 1.80 e é compatível com o MSRV 1.85 do Nexxus.
3. Não foi adotado `cosmic-text` 0.17+ nesta etapa porque as versões inspecionadas 0.17.2/0.19.0 declaram Rust 1.89; elevar silenciosamente o MSRV do projeto seria quebra de contrato.
4. SVG usa `resvg = 0.45.1` com `default-features = false`. A etapa precisa de SVG geométrico/simbólico; texto SVG e raster-images não são necessários ao contrato atual. A versão 0.45.1 declara MSRV 1.67.1 e usa `tiny-skia` 0.11.4.
5. FreeType/HarfBuzz C/FFI não são dependências diretas desta etapa. O stack selecionado satisfaz o contrato atual em Rust e reduz fronteiras unsafe/FFI no código Nexxus. Essa decisão pode ser reavaliada futuramente somente se medições ou compatibilidade demonstrarem necessidade real.
6. Hack permanece a família semântica padrão. O asset/licença da fonte pertence à Etapa 08 — Visual Assets; ausência do arquivo Hack no host não pode quebrar a UI Core, portanto o mecanismo de fallback continua funcional.

## Avaliação

- **modularidade:** renderer, display list e apresentação estão separados;
- **segurança:** `nexxus-ui` usa `#![forbid(unsafe_code)]`;
- **footprint:** features SVG não necessárias foram desabilitadas; medição real do harness integra a CI;
- **licenças das dependências diretas:** cosmic-text MIT OR Apache-2.0; resvg Apache-2.0 OR MIT; tiny-skia (transitiva/reexportada pelo resvg) BSD-3-Clause;
- **compatibilidade:** nenhuma dependência direta de GTK, Qt, X11 ou Wayland entra no crate.

## Fontes primárias consultadas

- upstream `pop-os/cosmic-text`, manifests e código da tag 0.16.0;
- upstream `linebender/resvg`, manifest/código da tag 0.45.1;
- upstream `RazrFalcon/tiny-skia`, manifest 0.11.4;
- bibliografia técnica oficial do Projeto Nexxus para fontes, SVG e renderização.
