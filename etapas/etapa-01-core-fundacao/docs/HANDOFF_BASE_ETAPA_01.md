# Handoff-base — Etapa 01

Este arquivo acompanha o snapshot. O SHA-256 e o commit que publica o próprio `.tar.gz` são registrados no handoff final depois da geração do arquivo, evitando autorreferência do hash.

## ETAPA ATUAL

Etapa 01 — Core e Fundação Arquitetural

## STATUS

**VALIDADA TECNICAMENTE / PRONTA PARA ENTREGA MATERIAL**

## ENTREGÁVEIS PRODUZIDOS

Workspace Rust 0.1.0 com `nexxus-core`, `nexxus-protocol`, `nexxus-config` e `nexxus-backend-api`; registry; lifecycle/rollback; Event Bus; paths XDG/runtime; IPC local; Unix Domain Socket privado; configuração TOML atômica; backend API abstrata; wrappers POSIX Arch/Debian; manifests; CI; ADRs; documentação e `Cargo.lock` gerado pelo Cargo.

## VALIDAÇÕES

GitHub Actions run `31974713059`: `archlinux-current = success` e `debian-trixie = success`. Build Release, rustfmt, Clippy `-D warnings`, testes, rustdoc, staging e auditoria POSIX aprovados.

## ESTADOS

- REVISADO: SIM
- COMPILADO: SIM
- TESTADO: SIM para os contratos exercitáveis da fundação
- EMPACOTADO NATIVO: N/A — não há payload instalável
- INSTALADO: N/A
- VALIDADO TECNICAMENTE: SIM

## GITHUB

- REPOSITORIO_GITHUB: `https://github.com/mintonogueira/nexxus-de`
- BRANCH: `main`
- PASTA_DA_ETAPA: `etapas/etapa-01-core-fundacao/`
- REVISAO_TECNICA_VALIDADA: `c714fe803fce32f59823d6d5ee7a217aa9d77d77`

## PRÓXIMA ETAPA

Não foi localizada definição autoritativa da etapa subsequente nas fontes consultadas para o fechamento.

- PASTA_PROXIMA_ETAPA: **EM ABERTO**
- STATUS_PROXIMA_ETAPA: **EM ABERTO**
- NOME_DA_NOVA_CONVERSA: **EM ABERTO**

Nenhuma Etapa 02 é inventada ou iniciada neste fechamento.
