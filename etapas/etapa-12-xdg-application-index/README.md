# Nexxus — Etapa 12 — XDG Application Index

**Status:** `EM_IMPLEMENTACAO`

A Etapa 12 entrega o índice unificado e dinâmico de aplicações que será consumido, por contrato, pelo Desktop Shell, Application Finder e Application Menu em etapas próprias.

## Entrega funcional

- leitura de arquivos `.desktop` em raízes XDG por precedência;
- inclusão explícita dos exports de aplicações Flatpak sem consultar o CLI do Flatpak;
- Desktop File ID canônico e mascaramento por `Hidden=true`;
- `Name`, `Exec`, `Icon`, `Categories`, `Keywords`, `NoDisplay`, `OnlyShowIn` e `NotShowIn` normalizados;
- categorias principais XDG e fallback `Other`;
- ícones oficiais preservados e fallback semântico fornecido pela Etapa 08;
- parser `Exec` que produz somente programa + argv e nunca chama shell;
- atualização dinâmica por eventos de filesystem com snapshots imutáveis e deltas;
- API comum de lookup, categorias, busca simples e assinatura de eventos;
- diagnósticos de entradas inválidas sem derrubar o serviço.

## Fronteiras

Esta etapa não implementa Menu visual, Desktop Shell, menu de contexto do desktop nem Application Finder visual.

## Crate

`crates/nexxus-xdg-application-index`

## Build

Os wrappers `scripts/build-install-arch.sh` e `scripts/build-install-debian.sh` são Shell POSIX `/bin/sh`, autoprovisionam as dependências do cenário, compilam/testam como usuário normal e preparam staging isolado. A entrega é biblioteca/serviço integrável (`NEXXUS_INSTALLABLE=0`); pacote nativo independente é N/A nesta etapa.
