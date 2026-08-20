# NEXXUS — ETAPA 15 — Panel Core

**Versão:** 0.1.0  
**Status:** IMPLEMENTADO / AGUARDANDO CI

Implementação do `nexxus-panel`, responsável pelo painel inferior fixo e pelo host leve de plugins do Nexxus.

## Entregue neste núcleo

- posição inferior calculada de forma determinística;
- altura configurável entre limites seguros;
- escala proporcional de ícones, padding, spacing e hit targets;
- zonas lógicas Start/Center/End sem bloquear movimentação;
- adicionar, remover, mover e reordenar instâncias;
- Plugin API versionada 1.0;
- load/unload com rollback quando shutdown falha;
- persistência versionada e atômica;
- contrato X11/EWMH para `_NET_WM_WINDOW_TYPE_DOCK`, `_NET_WM_STRUT` e `_NET_WM_STRUT_PARTIAL`;
- suíte de testes unitários do core.

## Fora do escopo preservado

Não implementa Application Menu, Dock/Task Buttons, plugins de rede/Bluetooth/áudio/notificações/energia/relógio nem Settings Panel. Esses componentes permanecem nas etapas próprias do Plano Mestre.
