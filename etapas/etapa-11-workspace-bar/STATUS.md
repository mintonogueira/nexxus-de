# STATUS — ETAPA 11

PROJETO: Nexxus  
ETAPA: 11 — Workspace Bar  
MÓDULO: nexxus-workspace-bar  
VERSÃO: 0.1.0  
STATUS: VALIDADO / ENTREGUE / PUBLICADO

## Implementado e validado

- modelo visual sincronizável com Workspace Manager;
- criação/remoção/rename/ativação incremental por `WorkspaceEvent`;
- layout suspenso e centralizado exclusivamente no monitor primário;
- DPI/escala por `ScaleFactor`;
- hit-testing e clique de workspace/settings;
- painter Nexxus UI, visual opaco e asset `preferences-workspaces`;
- adapter X11 com RandR, superfície `override_redirect` e input de mouse;
- wrappers POSIX Arch/Debian, validação X11/Xvfb e geração de entrega;
- ADR da decisão técnica da superfície X11 e seleção do monitor primário.

## Validação final

GitHub Actions run `32003637152`: Arch Linux, Debian Trixie e delivery concluídos com sucesso.

Commit validado publicado na `main`: `d866d91b723bc6747e08a7451be3ae6ef3799b94`.

## Entrega

- `entrega/Nexxus_Etapa11_Workspace_Bar_0.1.0.tar.gz`
- SHA-256: `8312e2674000130951b3c91c3d93660f23634f37532d15008371090823fb1ad6`
- handoff: `HANDOFF.md`

## Pendências da Etapa 11

Nenhuma pendência funcional ou de publicação dentro do escopo aprovado.
