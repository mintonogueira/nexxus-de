# Status — Etapa 03 — Session Runtime

Data-base: 2026-08-16

Estado atual: **EM IMPLEMENTAÇÃO / AGUARDANDO CI REAL**.

## Implementado nesta revisão

- workspace Rust e crate/binário `nexxus-session`;
- configuração e CLI de backend explícito;
- preflight XDG/runtime;
- endpoint IPC privado;
- lifecycle backend -> WM e shutdown reverso;
- rollback por contrato da fundação;
- testes unitários e de contrato;
- scripts POSIX e packaging nativo Arch/Debian.

## Pendente antes do fechamento

- CI Arch Linux e Debian;
- build release, rustfmt, Clippy, testes e rustdoc reais;
- geração/validação dos pacotes nativos;
- snapshot e SHA-256;
- handoff final;
- publicação do estado validado na `main`.
