# CI — Etapa 01

O workflow canônico está em `.github/workflows/etapa-01-core.yml`, único path reconhecido pelo GitHub Actions para esta automação compartilhada. Sua responsabilidade permanece rastreada à Etapa 01.

A matriz executa os wrappers oficiais da própria etapa em:

- Arch Linux current;
- Debian Trixie.

Cada job prepara apenas o mecanismo de privilégio mínimo, cria `nexxus-builder` e então executa o wrapper da distribuição como usuário não privilegiado. O próprio wrapper resolve as dependências ausentes e executa build release, `rustfmt --check`, Clippy com `-D warnings`, testes, rustdoc e staging.

Run de referência aprovado em 16/08/2026: `31974201820`, commit `42dc0a0713e4e21c772b3dac28b3edf47a0fab1a`.

O passo de exportação de fontes formatados existe apenas em falha no job Debian para facilitar diagnóstico de divergência de `rustfmt`; ele é ignorado em runs verdes.
