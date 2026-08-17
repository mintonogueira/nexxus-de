# ADR-001 — Modelo lógico de workspaces e persistência

**Status:** APROVADO COMO DECISÃO TÉCNICA INTERNA DA ETAPA 05  
**Data:** 2026-08-16

## Contexto

A especificação do Nexxus exige workspaces fixas/dinâmicas, MRU, placement inicial de aplicações, movimentação livre posterior e uma mesma workspace distribuível pelos monitores, sem vínculo rígido workspace-monitor.

## Decisão

1. `nexxus-workspaces` mantém somente identidade, ordem, tipo, nome, membership de janelas, workspace ativa, MRU, regras de placement e política de lifecycle dinâmico.
2. Nenhum identificador de monitor faz parte do modelo persistente ou runtime da workspace. A posição física continua expressa pela geometria da janela no Window Manager Core.
3. Regras de aplicação são consultadas somente em `assign_new_window`; `move_window` jamais reavalia a regra. Isso preserva a decisão normativa de não aprisionar aplicações.
4. A remoção de uma workspace com janelas primeiro realoca todas as janelas para uma workspace sobrevivente determinística e somente depois remove a workspace.
5. A política dinâmica expõe `keep-empty` e `remove-empty-inactive`; a escolha é configuração, não comportamento codificado de forma irreversível.
6. A configuração é persistida por `nexxus-config::TomlConfigStore`, utilizando envelope de schema versão 1, escrita atômica e limites já definidos na fundação.
7. Membership de janelas não é persistido. Retomada de processos/janelas pertence à Etapa 53 — Session State.
8. O crate utiliza `nexxus-wm::WindowId` e não importa APIs X11/Wayland. A adaptação EWMH do backend X11 deve consumir os eventos/estado lógico sem transferir responsabilidades do Workspace Manager para o backend.

## Consequências

- o módulo permanece reutilizável pelo futuro backend Wayland;
- a futura Workspace Bar pode consumir eventos sem conhecer X11;
- Alt+Tab poderá filtrar o MRU do WM pela membership da workspace atual;
- Super+Tab poderá consumir o MRU de workspaces desta etapa;
- Session State poderá restaurar membership posteriormente sem alterar o formato de configuração estática.

## Fontes técnicas relevantes

- EWMH: `_NET_NUMBER_OF_DESKTOPS`, `_NET_CURRENT_DESKTOP`, `_NET_DESKTOP_NAMES` e `_NET_WM_DESKTOP` para a futura adaptação X11;
- `nexxus-config` da Etapa 01 para persistência transacional;
- `nexxus-wm` da Etapa 02 para identidade backend-neutral de janelas.
