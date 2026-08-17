# NEXXUS — ETAPA 10 — Shortcuts Core

**Versão:** 0.1.0  
**Status:** VALIDADO EM BRANCH  
**Data:** 2026-08-17

## Escopo entregue

A etapa implementa a infraestrutura central de atalhos globais do Nexxus:

- command registry e binding registry;
- representação backend-neutral de teclas, modificadores e combinações;
- defaults aprovados pela documentação;
- captura de nova combinação;
- detecção de conflitos antes de persistir;
- dispatch desacoplado dos módulos consumidores;
- persistência TOML versionada pelo `nexxus-config`;
- adaptador inicial de grabs globais X11;
- base para teclas de multimídia e brilho;
- preservação explícita de `F11` para as aplicações.

A UI completa de configuração pertence à etapa de Settings pertinente e não é implementada aqui. O backend Wayland/GlobalShortcuts também permanece fora desta etapa.

## Contrato de dispatch

`Trigger -> CommandId -> CommandDescriptor -> CommandTarget`

O host que compõe a sessão implementa `ShortcutDispatchSink` e encaminha o alvo lógico ao módulo proprietário. Assim, esta etapa não incorpora estado interno de WM, Workspace Manager, Tiling Engine, launchers, sessão, captura, áudio ou brilho.

## Super isolado

`Super` é modelado como `ModifierTap`: só dispara quando o modificador é pressionado e solto sem participar de outra combinação. `Super+F`, `Super+T` e demais chords não produzem um segundo acionamento do menu ao liberar `Super`.

## X11

O adaptador usa `x11rb` 0.14. Ele consulta o mapa real do servidor, descobre os grupos de Alt/Super e cria passive grabs tolerando estados de Caps Lock, Num Lock e Scroll Lock. Erro de grab produz falha explícita e rollback.

## Validação registrada

GitHub Actions run `32000106232`: Arch Linux, Debian Trixie e delivery concluíram com sucesso. Foram executados release build, rustfmt restrito ao módulo, Clippy `-D warnings`, 15 testes unitários, teste X11 real sob Xvfb, rustdoc, staging e geração do snapshot.

## Build e distribuição

Os wrappers de orquestração permanecem separados e 100% POSIX:

- `scripts/build-install-arch.sh`;
- `scripts/build-install-debian.sh`.

Eles autoprovisionam dependências, compilam como usuário normal, testam e preparam staging isolado.

O módulo é uma biblioteca/runtime integrável (`NEXXUS_INSTALLABLE=0`); não existe payload executável independente nesta etapa, portanto pacote binário/instalação são `N/A` em vez de fabricar um pacote vazio.

Consulte `HANDOFF.md` para contratos, validações, limites, artefatos e continuidade.
