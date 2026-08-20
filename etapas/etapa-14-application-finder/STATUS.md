# STATUS — Etapa 14 — Application Finder

- **Projeto:** Nexxus
- **Etapa:** 14 — Application Finder
- **Módulo:** `nexxus-app-finder`
- **Versão:** 0.1.0
- **Branch de implementação:** `etapa-14-application-finder-impl`
- **Estado:** `EM_ANDAMENTO_BLOQUEADO_PARCIALMENTE`

## Implementado nesta branch

- ranking incremental determinístico;
- busca por Name/Keywords/Categories/Desktop ID;
- suporte interno e testes de busca por Comment;
- fuzzy subsequence de baixa prioridade;
- UI compacta em `nexxus-ui`;
- foco inicial no campo de pesquisa;
- ArrowUp/ArrowDown, Enter, Escape e mouse;
- integração lógica com `LauncherAction::ApplicationFinder`;
- atualização da busca quando a geração do índice muda;
- execução shell-free por argv;
- ativação D-Bus `org.freedesktop.Application`;
- scripts POSIX Arch/Debian e CI da etapa.

## Bloqueio

**GitHub issue #16:** o contrato público da Etapa 12 não expõe `Comment`. A validação integral do requisito "busca por comment" não pode ser declarada até o módulo proprietário fornecer o metadado.

Nenhuma modificação na Etapa 12 foi feita nesta conversa.
