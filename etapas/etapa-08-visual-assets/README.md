# Nexxus — Etapa 08 — Visual Assets

**Status:** `EM ANDAMENTO`

A Etapa 08 formaliza os recursos visuais próprios do Nexxus sem antecipar Window Chrome, Panel, Settings ou temas GTK/Qt. O payload runtime é composto somente por assets e manifests; a integração gráfica continua consumindo a infraestrutura entregue pela Etapa 07.

## Conteúdo

- 82 ícones simbólicos SVG 24×24, organizados por contexto semântico;
- 10 wallpapers vetoriais dark 1920×1080;
- manifesto versionado e tabelas de fallback;
- crate `nexxus-assets` com catálogo, resolução de fallback e recoloração do token simbólico;
- dependência de runtime da fonte Hack provida pela distribuição, sem vendorizar arquivos de fonte;
- empacotamento nativo Arch Linux e Debian;
- validação de SVG/XML, rasterização em múltiplas escalas e auditoria de conteúdo ativo/externo.

## Contrato visual

SVGs simbólicos próprios usam `#FFFFFF` como token canônico de cor. O consumidor pode substituí-lo pela cor semântica vigente antes de enviar os bytes ao renderer SVG da Etapa 07. Ícones oficiais declarados por aplicações externas nunca passam por esse mecanismo.

## Runtime

Os dados são instalados em `/usr/share/nexxus/assets`. O package runtime depende de Hack pelo pacote oficial da distribuição.

## Fronteiras preservadas

Branding/logo final, Window Chrome, temas GTK/Qt, efeitos e animações permanecem fora desta etapa.
