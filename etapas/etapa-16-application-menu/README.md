# NEXXUS — Etapa 16 — Application Menu

Implementa o menu de aplicações do Nexxus como consumidor do XDG Application Index e plugin do Panel Core.

## Escopo implementado

- abertura/fechamento e toggle lógico do menu;
- busca instantânea delegada ao índice XDG comum;
- seções Favoritos, Recentes, Todos e categorias XDG;
- favoritos determinísticos e recentes em ordem MRU;
- modos List/Grid e tamanhos Small/Medium/Large;
- preservação do ícone oficial/fallback fornecido pela Etapa 12;
- geração de `LaunchCommand` shell-free a partir do `ExecTemplate` validado;
- integração com `PanelPlugin` API 1.0 do Panel Core;
- testes de busca, favoritos, recentes, lançamento e lifecycle do plugin;
- pipelines POSIX independentes para Arch Linux e Debian.

## Dependências

- Etapa 12 — `nexxus-xdg-application-index`;
- Etapa 15 — `nexxus-panel`.

A branch desta etapa é empilhada sobre `etapa-15-panel-core-impl` porque a Etapa 15 ainda não está integrada à `main`. Nenhum código da Etapa 15 é reimplementado aqui.

## Fora do escopo

- Dock/Task Buttons;
- Settings Panel completo;
- reimplementação do parser/monitor XDG;
- implementação de backend gráfico X11/Wayland.
