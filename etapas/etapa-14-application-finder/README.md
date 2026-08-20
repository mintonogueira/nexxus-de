# NEXXUS — Etapa 14 — Application Finder

Implementação do `nexxus-app-finder`, finder compacto e backend-neutral acionado pelo comando lógico `nexxus.launcher.application-finder` (`Super+F` no default da Etapa 10).

## Responsabilidade

- consumir o catálogo e as gerações dinâmicas da Etapa 12;
- fazer busca incremental e ranking específico do Finder;
- compor a superfície com `nexxus-ui`;
- oferecer navegação por teclado e mouse;
- executar a aplicação selecionada sem shell, usando `ExecTemplate`/argv ou `org.freedesktop.Application`.

O módulo **não** descobre nem reparsa `.desktop`, não implementa menu Whisker, não busca arquivos e não altera os módulos 07, 10 ou 12.

## Busca

O ranking favorece, em ordem geral, nome exato/prefixo, palavras do nome, keywords, comment, categories e Desktop File ID. Consultas com múltiplos termos usam semântica AND. A ordenação é determinística.

## Bloqueio de contrato — `Comment`

O requisito da Etapa 14 inclui busca por `Comment`, mas o `ApplicationRecord` público da Etapa 12 ainda não expõe esse campo. Para preservar isolamento de módulos, a Etapa 14 **não** criou um segundo parser de `.desktop`.

O algoritmo e seus testes já suportam `Comment` por `CommentProvider`, porém a integração XDG final aguarda o ajuste registrado no GitHub issue #16. Até esse contrato ser fornecido pelo módulo proprietário, a Etapa 14 permanece `EM_ANDAMENTO_BLOQUEADO_PARCIALMENTE`.

## Build

Entradas obrigatórias Shell 100% POSIX:

```sh
sh ./scripts/build-install-arch.sh
sh ./scripts/build-install-debian.sh
```

O módulo é runtime integrável (`NEXXUS_INSTALLABLE=0`), portanto não fabrica pacote executável vazio.
