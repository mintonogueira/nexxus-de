# Nexxus — Etapa 07 — Nexxus UI Core

**Status:** `EM ANDAMENTO`

Esta etapa implementa `nexxus-ui`, a camada gráfica própria e backend-neutral do Nexxus. O crate não usa GTK/Qt e não contém tipos concretos X11/Wayland. Widgets produzem uma `DisplayList`; o `SoftwareRenderer` transforma essa lista em um frame RGBA8 que um adapter gráfico pode apresentar.

## Escopo implementado

- geometria lógica/física e escala fracionária;
- paleta dark opaca, métricas e Hack como família padrão;
- display list com retângulos, bordas, clipping, texto, imagem e SVG;
- renderer software independente de backend;
- shaping/rasterização de texto com fallback de fontes;
- widgets base: containers, button, toggle, checkbox, text field, list, scroll, menu, popup e tabs;
- foco, mouse, teclado, hit-testing e mensagens semânticas;
- árvore semântica preparada para futuro bridge de acessibilidade;
- harness `nexxus-ui-demo` que gera frame PPM;
- testes de layout/input/escala/renderização;
- medição automática inicial de RSS do harness.

## Fronteiras preservadas

Painel, menu de aplicações, Settings, File Manager, Window Chrome, Visual Assets e backend Wayland completo não são implementados aqui. A integração X11 recebe pixels/contratos abstratos, sem mover código do Backend X11 para esta etapa.
