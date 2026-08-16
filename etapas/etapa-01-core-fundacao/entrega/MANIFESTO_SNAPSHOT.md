# Manifesto Final da Entrega — Etapa 01

- Projeto: Nexxus
- Etapa: 01 — Core e Fundação Arquitetural
- Versão: `0.1.0`
- Data: `2026-08-16`
- Repositório: `https://github.com/mintonogueira/nexxus-de`
- Branch: `main`
- Snapshot: `Nexxus_Etapa01_Core_0.1.0.tar.gz`
- SHA-256: `06928c110a20b65a0019e3dbe856b35ada3310f772d3e93c4b2874d54b361b7d`
- Snapshot source commit: `057fcc7c2889e299792d46b7fcbd74a304f8a900`
- GitHub Actions run: `31975039874`
- Commit que publica os artefatos na `main`: `7da0a7f837b67933461325b225806a37c293061c`

## Conteúdo

O snapshot contém:

- `etapas/etapa-01-core-fundacao/` no estado do commit-fonte;
- `.github/workflows/etapa-01-core.yml`;
- `LICENSE`;
- `SNAPSHOT_BUILD_INFO.txt` gerado pela CI, contendo `SNAPSHOT_SOURCE_COMMIT`, `GITHUB_RUN_ID`, versão e identificação da etapa.

## Validação

- Arch Linux current: aprovado.
- Debian Trixie: aprovado.
- Geração do snapshot: aprovada.
- `tar -tzf`: aprovado.
- Verificação de caminhos absolutos/`..`: aprovada.
- SHA-256 recalculado após download do artefato: coincide com o valor acima.

## Exclusões

Não foram incluídos `.git/`, `target/`, `.build/`, staging, caches, logs transitórios, credenciais, tokens ou arquivos específicos da máquina de build.

Pacote binário nativo e instalação são N/A nesta etapa porque não existe payload de runtime instalável.
