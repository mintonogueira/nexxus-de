# ADR-001 — Ranking do Application Finder

**Status:** IMPLEMENTADO NA ETAPA 14  
**Escopo:** somente `nexxus-app-finder`

## Decisão

O Finder mantém uma projeção imutável da geração atual do XDG Application Index e executa ranking próprio, sem alterar a busca comum da Etapa 12.

A consulta é normalizada com `trim()` e lowercase Unicode do Rust. Termos separados por espaço usam semântica AND. Cada termo recebe a melhor pontuação encontrada entre:

1. Name exato;
2. Name prefixo;
3. início de palavra em Name;
4. ocorrência em Name;
5. Keywords;
6. Comment;
7. Categories;
8. Desktop File ID;
9. subsequência fuzzy do Name, com peso baixo.

Empates são resolvidos por nome normalizado e Desktop File ID, garantindo resultado determinístico.

## Motivo

A Etapa 12 explicitamente deixou fuzzy/ranking específico para o Application Finder. A separação evita contaminar o catálogo comum com política de UX de um consumidor específico.

## Segurança e footprint

- nenhuma thread adicional para busca;
- nenhum parser `.desktop` na Etapa 14;
- nenhuma chamada a shell;
- cópia pequena de metadados para evitar retenção de locks do serviço do índice.

## Dependência pendente

A Etapa 12 ainda não fornece `Comment` no `ApplicationRecord`. O `CommentProvider` é um seam temporário para permitir testar o algoritmo sem invadir a responsabilidade do módulo proprietário. Issue de integração: #16.
