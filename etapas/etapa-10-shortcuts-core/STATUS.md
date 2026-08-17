# Estado da Etapa 10

- **Projeto:** Nexxus
- **Etapa:** 10 — Shortcuts Core
- **Módulo:** `nexxus-shortcuts`
- **Versão:** 0.1.0
- **Status:** VALIDADO E PUBLICADO
- **Branch de construção:** `etapa-10-shortcuts-core-impl`
- **Integração inicial na main:** `cbea8c233557b3643ac4f9405869626afe283574`
- **Validação final:** GitHub Actions run `32000339497` — Arch Linux, Debian e delivery: SUCCESS

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

Etapa 10 publicada na `main`; nenhuma pendência de publicação permanece para este módulo.
