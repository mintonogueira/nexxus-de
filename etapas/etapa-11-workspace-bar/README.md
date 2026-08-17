# NEXXUS — ETAPA 11 — Workspace Bar

**Status:** VALIDADO / ENTREGUE / PUBLICADO  
**Versão:** 0.1.0  
**Módulo:** `nexxus-workspace-bar`

Implementação validada da barra superior suspensa de workspaces definida pela documentação normativa do Nexxus.

## Responsabilidades entregues

- espelhar a ordem, nomes e workspace ativo do `nexxus-workspaces`;
- reagir a `Created`, `Removed`, `Renamed` e `Activated` sem duplicar a autoridade do Workspace Manager;
- trocar workspace por clique;
- expor `OpenWorkspaceSettings` sem implementar o módulo Settings nesta etapa;
- desenhar a UI exclusivamente com `nexxus-ui` e Visual Assets;
- existir somente no monitor primário, com geometria lógica/HiDPI;
- fornecer adapter X11 inicial com RandR e superfície `override_redirect`;
- manter superfícies opacas, sem animação, blur, sombra ou transparência decorativa.

## Fora do escopo preservado

Settings de workspaces, painel inferior, backend Wayland definitivo e alteração do Workspace Manager.

## Validação

GitHub Actions run `32003637152`: Arch Linux, Debian Trixie e delivery concluídos com sucesso.

```sh
sh ./scripts/check-posix.sh
sh ./scripts/check.sh
```

Consulte `HANDOFF.md` para arquitetura, contratos, evidências, artefatos e continuidade.
