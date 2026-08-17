# Nexxus — Etapa 07 — Nexxus UI Core

**Status:** `VALIDADO / ENTREGUE / PUBLICADO`

Esta etapa implementa `nexxus-ui`, a camada gráfica própria e backend-neutral do Nexxus. O crate não usa GTK/Qt e não contém tipos concretos X11/Wayland. Widgets produzem uma `DisplayList`; o `SoftwareRenderer` transforma essa lista em um frame RGBA8 que um adapter gráfico pode apresentar.

## Escopo implementado

- geometria lógica/física e escala fracionária;
- paleta dark opaca, métricas e Hack como família padrão;
- display list com superfícies/frame, retângulos, bordas, clipping, texto, imagem e SVG;
- renderer software independente de backend;
- shaping/rasterização de texto com fallback de fontes;
- widgets base: containers, button, toggle, checkbox, text field, list, scroll, menu, popup e tabs;
- foco, mouse, teclado, hit-testing e mensagens semânticas;
- árvore semântica preparada para futuro bridge de acessibilidade;
- harness `nexxus-ui-demo` que gera frame PPM;
- testes de layout/input/escala/renderização;
- medição automática inicial de RSS do harness.

## Validação real

Workflow da branch `31990830331`:

- Arch Linux current: **SUCCESS**;
- Debian Trixie: **SUCCESS**;
- delivery/snapshot: **SUCCESS**.

Workflow pós-merge em `main` `31991101667`:

- Arch Linux current: **SUCCESS**;
- Debian Trixie: **SUCCESS**;
- delivery: **SKIPPED por design**, porque o snapshot validado já estava versionado.

Também foram aprovados 8 testes Rust, fmt, Clippy com `-D warnings`, rustdoc e auditorias de fronteira. O RSS máximo medido do harness foi **6376 KiB**.

## Fronteiras preservadas

Painel, menu de aplicações, Settings, File Manager, Window Chrome, Visual Assets e backend Wayland completo não são implementados aqui. A integração X11 recebe pixels/contratos abstratos, sem mover código do Backend X11 para esta etapa.

O asset/licença da fonte Hack permanece responsabilidade da Etapa 08 — Visual Assets. A Etapa 07 usa Hack como família semântica padrão e mantém fallback de fonte para que a infraestrutura gráfica não dependa da presença antecipada desse asset.
