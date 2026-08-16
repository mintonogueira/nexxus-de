# Manifesto do Snapshot — Etapa 01

Projeto: Nexxus  
Etapa: 01 — Core e Fundação Arquitetural  
Versão: 0.1.0  
Data: 2026-08-16  
Repositório: `https://github.com/mintonogueira/nexxus-de`  
Branch: `main`  
Revisão técnica validada: `c714fe803fce32f59823d6d5ee7a217aa9d77d77`  
CI: run `31974713059` — Arch Linux current e Debian Trixie aprovados.

## Conteúdo

- `etapas/etapa-01-core-fundacao/`
- `.github/workflows/etapa-01-core.yml`
- `LICENSE`

## Exclusões

`.git/`, `target/`, `.build/`, staging, logs transitórios, caches, credenciais, tokens e arquivos específicos da máquina de build.

## Build/instalação

A Etapa 01 entrega bibliotecas e contratos; não existe payload de runtime instalável. Pacote nativo e instalação permanecem N/A, sem pacote vazio.

O SHA-256 do `.tar.gz` é registrado externamente no handoff final e no arquivo `.sha256`, evitando autorreferência do hash dentro do próprio snapshot.
