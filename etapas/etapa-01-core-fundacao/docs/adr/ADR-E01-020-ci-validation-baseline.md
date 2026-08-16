# ADR-E01-020 — Baseline de validação da fundação

**Status:** aceito — 2026-08-16

Uma revisão da fundação só é considerada compilada/testada quando os jobs canônicos Arch Linux current e Debian Trixie executarem com sucesso os wrappers da própria etapa, incluindo build Release, `cargo fmt --check`, Clippy `-D warnings`, testes, rustdoc e staging. O `Cargo.lock` versionado também precisa ser aceito nos dois cenários.

## Evidência final 0.1.0

Run `31974713059`, commit `c714fe803fce32f59823d6d5ee7a217aa9d77d77`: ambos os jobs concluíram com sucesso usando o `Cargo.lock` gerado pelo Cargo.

Uma execução intermediária (`31974568491`) detectou checksum incorreto em um lockfile transcrito manualmente. O arquivo foi substituído pelo artefato efetivamente gerado pelo Cargo e toda a matriz foi reexecutada antes do fechamento.
