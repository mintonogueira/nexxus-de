# Changelog — Etapa 13

## 0.1.0 — 2026-08-17

- implementação inicial do Desktop Shell;
- persistência de wallpaper e launchers;
- menu de contexto e categorias de aplicações;
- integração com Shortcuts Core e XDG Application Index;
- adapter X11 inicial com RandR e EWMH Desktop window type;
- scripts POSIX Arch/Debian, testes e CI da etapa;
- resolver Cargo v3 e `zbus = 5.13.2` fixado para preservar MSRV Rust 1.85;
- correções de propagação de erros no adapter X11 e eliminação de warnings Clippy;
- validação concluída em Arch Linux e Debian Trixie, incluindo smoke test X11/Xvfb;
- delivery automatizado com `Cargo.lock`, snapshot `.tar.gz` e SHA-256.
