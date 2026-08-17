# ADR-001 — Contrato de Visual Assets da Etapa 08

**Status:** APROVADO INTERNAMENTE — decisão técnica da etapa

## Contexto

A documentação normativa exige SVGs simbólicos escaláveis e recoloríveis, enquanto a Etapa 07 já aceita bytes SVG via `DrawCommand::Svg` sem parâmetro de tint. Alterar a API da Etapa 07 nesta conversa violaria o isolamento por etapa.

## Decisão

1. SVGs simbólicos próprios usam `#FFFFFF` como token canônico de cor.
2. `nexxus-assets::recolor_symbolic_svg` substitui somente esse token por `#RRGGBB` antes de entregar os bytes ao renderer existente.
3. Ícones externos declarados por `.desktop` são representados como `ApplicationIcon::External` e nunca são recoloridos.
4. Fallbacks só são usados quando não existe ícone oficial da aplicação.
5. Hack não é vendorizada. O pacote `nexxus-visual-assets` depende do pacote oficial da distribuição.
6. Wallpapers e SVGs próprios não usam filtros, opacidade decorativa, recursos externos ou conteúdo ativo.

## Motivo

A solução satisfaz o requisito de recoloração, mantém os assets independentes de GTK/Qt, evita modificar a Etapa 07 e mantém o payload simples, auditável e de baixo footprint.

## Consequências

- futuros consumidores podem aplicar qualquer paleta sem duplicar arquivos SVG;
- o renderer existente permanece inalterado;
- ícones de aplicações externas preservam autoria e identidade visual original;
- o contrato pode ser consumido pela Etapa 09 e componentes posteriores sem acoplamento ao filesystem físico.
