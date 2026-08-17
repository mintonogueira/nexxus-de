# ADR-001 — Superfície X11 e monitor primário da Workspace Bar

**Status:** ACEITA  
**Data:** 2026-08-17  
**Escopo:** Etapa 11 — Workspace Bar

## Contexto

A documentação normativa exige uma barra superior suspensa, dinâmica, clicável e visível somente no monitor principal. O Workspace Manager continua sendo a autoridade sobre workspaces e não associa workspace rigidamente a monitor.

## Decisão

- a Workspace Bar mantém somente um modelo de apresentação derivado de `nexxus-workspaces`;
- eventos `Created`, `Removed`, `Renamed` e `Activated` atualizam esse modelo sem duplicar a autoridade do Workspace Manager;
- o adapter X11 usa uma janela `override_redirect` própria para que a barra não seja tratada como janela normal de aplicação;
- RandR é consultado apenas para descobrir a geometria do monitor primário e posicionar a barra;
- a barra não cria associação workspace↔monitor e não altera a política multi-monitor do core;
- a ação de Settings é exposta ao coordenador/runtime, sem implementar Settings de Workspaces nesta etapa;
- a barra não drena unilateralmente a fila do Workspace Manager: eventos destinados a múltiplos consumidores devem ser distribuídos pelo coordenador, ou o consumidor pode sincronizar por snapshot completo.

## Motivação

A decisão mantém as responsabilidades das Etapas 04, 05, 07 e 08 isoladas, preserva a regra de monitor primário sem contaminar o modelo de workspaces e permite que um backend Wayland futuro substitua somente o adapter de superfície/topologia.

## Consequências

- o runtime de sessão deverá instanciar o componente e encaminhar ações/eventos entre módulos;
- o futuro backend Wayland deverá implementar adapter equivalente sem alterar o modelo, layout, input ou painter da barra;
- mudanças de monitor primário exigem chamada de atualização de topologia pelo runtime quando o backend sinalizar alteração;
- Settings de Workspaces permanece fora do escopo e será desenvolvido em sua etapa própria.

## Validação

A Etapa 11 possui testes de modelo, eventos incrementais, hit-testing, HiDPI, seleção do monitor primário e criação real da superfície X11 sob Xvfb. Os pipelines Arch Linux e Debian executam a suíte e o smoke test X11.
