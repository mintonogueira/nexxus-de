# STATUS — Etapa 05 — Workspace Manager

- **Projeto:** Nexxus
- **Etapa:** 05 — Workspace Manager
- **Módulo:** `nexxus-workspaces`
- **Versão:** 0.1.0
- **Status:** EM IMPLEMENTAÇÃO
- **Repositório:** `https://github.com/mintonogueira/nexxus-de`
- **Branch:** `etapa-05-workspace-manager-impl`

## Implementado

- modelo backend-neutral de workspace;
- workspaces fixas e dinâmicas;
- workspace ativa e MRU determinístico;
- membership e movimentação manual de janelas;
- placement inicial por `application_id` sem aprisionamento posterior;
- remoção segura sem perda de janelas;
- política configurável de retenção de workspaces dinâmicas vazias;
- persistência TOML versionada via `nexxus-config`;
- eventos create/remove/rename/activate/move-window/forget-window;
- testes de lifecycle, MRU, placement, remoção e persistência;
- wrappers POSIX separados para Arch Linux e Debian;
- auditorias de POSIX e neutralidade de backend;
- geração de snapshot e SHA-256 preparada.

## Validação em andamento

A primeira execução de CI (`31985696895`) detectou corretamente uma falha na própria auditoria POSIX: o verificador examinava seu arquivo e identificava como bashism a expressão regular usada para detectar bashisms. A auditoria foi corrigida para não analisar a si própria; nenhuma regra POSIX foi relaxada.

## Pendente desta etapa

- revalidação real em CI Arch Linux e Debian após a correção da auditoria;
- correções decorrentes da CI, se necessárias;
- geração/versionamento do snapshot validado;
- handoff final;
- publicação final na `main`.

## Fora do escopo preservado

Tiling Engine, Workspace Bar, Settings gráfico de workspaces, Session State completo e backend Wayland/XWayland.
