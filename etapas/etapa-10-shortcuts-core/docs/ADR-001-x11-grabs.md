# ADR-001 — Grabs X11 sem contaminar o Shortcuts Core

**Status:** APROVADO INTERNAMENTE NA ETAPA  
**Data:** 2026-08-17

## Problema

A Etapa 10 precisa oferecer grabs globais X11 iniciais, mas o registry, a persistência e o dispatch devem permanecer independentes de backend para permitir Wayland posteriormente.

Também existe um caso especial: `Super` isolado abre o menu, ao mesmo tempo em que `Super` participa de vários chords.

## Decisão

O crate mantém toda a semântica de shortcuts em tipos backend-neutral. O código X11 fica isolado em `x11.rs` e traduz o registry para passive grabs concretos.

O adaptador consulta `GetKeyboardMapping`, consulta `GetModifierMapping`, descobre os masks reais de Alt e Super, adiciona combinações equivalentes com lock modifiers, instala grabs no root window e faz rollback de todos os grabs criados quando um deles falha.

Para modifier tap, o próprio modificador é passivamente agarrado. Isso inicia o active keyboard grab ao pressioná-lo. Chords que contêm esse modifier tap não instalam grabs redundantes: seus eventos são observados no active grab e a semântica continua sendo decidida pelo recognizer/dispatcher backend-neutral.

## Consequências

- `F11` nunca entra na lista de grabs globais.
- O core não conhece XIDs, keycodes ou `ModMask`.
- Falha de grab é explícita; não existe sobrescrita silenciosa.
- Wayland poderá fornecer outro adaptador sem alterar registry/persistência/dispatch.
- Teclas ausentes no mapa físico atual não impedem a inicialização; simplesmente não geram grab naquele servidor.
- Nomes de tecla ainda desconhecidos pelo adaptador X11 retornam erro explícito, sem serem reinterpretados.

## Dependências

Nenhuma crate nova de alto nível foi adicionada. O adaptador reutiliza `x11rb` 0.14, já adotado pelo Backend X11 do Nexxus. A persistência reutiliza `nexxus-config`; o identificador de tile-fit reutiliza o contrato da Etapa 06.
