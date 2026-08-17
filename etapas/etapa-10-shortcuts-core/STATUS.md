# Estado da Etapa 10

- **Projeto:** Nexxus
- **Etapa:** 10 — Shortcuts Core
- **Módulo:** `nexxus-shortcuts`
- **Versão:** 0.1.0
- **Status:** VALIDADO EM BRANCH
- **Branch:** `etapa-10-shortcuts-core-impl`
- **Validação:** GitHub Actions run `32000106232` — Arch Linux, Debian e delivery: SUCCESS

## Implementado e validado

- modelo backend-neutral de triggers;
- reconhecimento de modifier tap e chords;
- registry de comandos e bindings;
- defaults normativos;
- F11 nunca global;
- conflito e rebind transacional;
- persistência versionada;
- dispatch por contrato;
- adaptador inicial de grabs X11 com rollback;
- 15 testes unitários;
- teste de grabs X11 real em Xvfb;
- wrappers POSIX Arch/Debian com autoprovisionamento;
- release build, rustfmt, Clippy `-D warnings` e rustdoc;
- snapshot `.tar.gz` e SHA-256.

## Distribuição

`NEXXUS_INSTALLABLE=0`: biblioteca/runtime integrável sem payload executável independente. Pacote nativo e instalação: `N/A` nesta etapa.

## Publicação

Pendente apenas a integração final do estado validado na branch canônica `main`.
