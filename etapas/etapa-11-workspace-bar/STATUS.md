# STATUS — ETAPA 11

PROJETO: Nexxus  
ETAPA: 11 — Workspace Bar  
MÓDULO: nexxus-workspace-bar  
VERSÃO: 0.1.0  
STATUS: EM_ANDAMENTO

## Implementado

- modelo visual sincronizável com Workspace Manager;
- criação/remoção/rename/ativação incremental por `WorkspaceEvent`;
- layout suspenso e centralizado exclusivamente no monitor primário;
- DPI/escala por `ScaleFactor`;
- hit-testing e clique de workspace/settings;
- painter Nexxus UI, visual opaco e asset `preferences-workspaces`;
- adapter X11 com RandR, superfície override-redirect e input de mouse;
- wrappers POSIX Arch/Debian, validação e geração de entrega.

## Pendente

- validação CI real Arch/Debian/Xvfb;
- correções decorrentes da validação;
- geração do snapshot final, SHA-256 e handoff;
- merge/publicação na `main`.
