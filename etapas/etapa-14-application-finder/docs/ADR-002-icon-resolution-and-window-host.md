# ADR-002 — Ícones de aplicações e contrato da janela do Finder

**Status:** IMPLEMENTADO NA ETAPA 14  
**Escopo:** somente `nexxus-app-finder`

## Contexto normativo

A Etapa 14 exige janela própria compacta e resultados com ícone oficial/fallback, mas suas dependências de entrada são exclusivamente as Etapas 07, 10 e 12. Portanto, esta etapa não deve acoplar o Finder a tipos concretos do backend X11/Wayland nem reimplementar o XDG Application Index.

## Decisão — janela

`FinderWindowSpec` descreve título, tamanho preferencial compacto e tamanho mínimo. `FinderWindowRequest` converte abertura/fechamento do controller em `ShowAndFocus`/`Hide` para o host da sessão/backend.

A janela permanece backend-neutral: o host gráfico cria o toplevel real conforme o backend ativo. Isso preserva as dependências formais da etapa e permite que `Super+F` gere uma solicitação inequívoca de mostrar e focar o Finder.

## Decisão — ícones

O Finder consome exclusivamente `IconReference` produzido pela Etapa 12:

- `ExternalPath`: preserva o caminho oficial quando o arquivo é PNG/SVG válido;
- `ExternalName`: busca em raízes XDG no fallback `hicolor` e em `pixmaps`;
- `NexxusFallback`: resolve o `relative_path` já escolhido pela Etapa 12;
- falha/corrupção/artefato não suportado: tenta `mimetypes/application-x-generic.svg` do pacote Nexxus.

O valor `Icon=` não é reparsado nesta etapa.

A resolução segue a regra da Desktop Entry Specification de usar caminho absoluto diretamente e lookup de tema para nomes, e usa `hicolor` como baseline determinístico porque o Nexxus ainda não possui nesta etapa uma política de tema de ícones de aplicações selecionável pelo usuário.

Fontes primárias consultadas:

- Freedesktop Desktop Entry Specification — `Icon` e `Comment`: https://specifications.freedesktop.org/desktop-entry/latest/recognized-keys.html
- Freedesktop Icon Theme Specification — lookup e fallback `hicolor`: https://specifications.freedesktop.org/icon-theme/latest/

## Formatos e footprint

SVG é enviado diretamente como `DrawCommand::Svg`. PNG usa `image = 0.25.9` fixado exatamente, com `default-features = false` e somente feature `png`. O CI da etapa confirmou essa versão como compatível com o MSRV Rust 1.85. XPM legado não recebe decoder novo nesta etapa; se for a única representação disponível, o Finder usa o fallback Nexxus.

Os gráficos resolvidos são cacheados por path. Arquivos acima de 8 MiB são recusados antes da leitura para evitar consumo indevido por ícones anômalos.

## Isolamento preservado

- nenhum parser `.desktop` adicional;
- nenhuma alteração nas Etapas 07, 10 ou 12;
- nenhum tipo X11/Wayland no crate;
- nenhum GTK/Qt;
- nenhuma política nova de tema global de aplicações.
