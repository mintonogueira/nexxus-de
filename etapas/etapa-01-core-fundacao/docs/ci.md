# CI — Etapa 01

Baseline final da revisão técnica 0.1.0:

- Run: `31974713059`
- Commit: `c714fe803fce32f59823d6d5ee7a217aa9d77d77`
- `archlinux-current`: success
- `debian-trixie`: success

Os wrappers oficiais foram executados como usuário não privilegiado e concluíram build Release, rustfmt, Clippy com `-D warnings`, testes, rustdoc e staging. O `Cargo.lock` presente nessa revisão foi gerado pelo Cargo e aceito nos dois cenários.
