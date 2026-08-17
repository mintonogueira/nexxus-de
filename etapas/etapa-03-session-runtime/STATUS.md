# Status — Etapa 03 — Session Runtime

Data-base: 2026-08-16

Estado atual: **VALIDADO_TECNICAMENTE / AGUARDANDO FECHAMENTO E PUBLICAÇÃO NA `main`**.

## Implementado

- workspace Rust e crate/binário `nexxus-session`;
- seleção explícita de backend `x11`/`wayland`, sem fallback silencioso;
- configuração versionada sobre `nexxus-config`;
- preflight XDG/runtime e endpoint IPC privado;
- lifecycle backend -> WM com shutdown reverso e rollback por contrato da fundação;
- estado mínimo de sessão para diagnóstico;
- testes de seleção, bootstrap, IPC, rollback e shutdown;
- scripts Shell 100% POSIX separados para Arch Linux e Debian;
- packaging nativo Arch Linux e Debian;
- geração automatizada de snapshot `.tar.gz` e SHA-256.

## Validação técnica

O workflow da Etapa 03 executa, nos dois cenários homologados desta etapa:

- auditoria POSIX dos wrappers;
- `cargo build --workspace --release`;
- `cargo fmt --all -- --check`;
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`;
- `cargo test --workspace --all-features`;
- `cargo doc` com `RUSTDOCFLAGS='-D warnings'`;
- staging isolado;
- geração e validação do pacote nativo;
- instalação do pacote efetivamente gerado;
- `nexxus-session --backend=x11 --check`;
- `nexxus-session --backend=wayland --check`;
- geração e teste do snapshot de entrega.

A evidência final de validação é o workflow correspondente ao commit de entrega desta etapa; o fechamento só será marcado como **PUBLICADO/ENTREGUE** depois que o estado final estiver na branch canônica `main`, com handoff e referência Git registrados.

## Limite intencional

Backend X11/Wayland concreto não pertence à Etapa 03. O runtime fornece o contrato de integração e falha explicitamente quando o backend selecionado não possui implementação concreta integrada, sem substituir a escolha do usuário por outro backend.
