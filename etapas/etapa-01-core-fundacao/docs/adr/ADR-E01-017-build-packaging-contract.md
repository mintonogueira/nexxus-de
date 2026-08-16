# ADR-E01-017 — Contrato comum de build e empacotamento

**Status:** aceito — 2026-08-16

## Decisão

A Etapa 01 estabelece dois entrypoints independentes, `build-install-arch.sh` e `build-install-debian.sh`, ambos Shell 100% POSIX. Dependências são declaradas por distribuição no manifesto da etapa. Build/testes ocorrem como usuário normal; elevação é pontual e exclusiva ao gerenciador nativo.

O pipeline sempre exercita preflight, dependências, build, testes e staging. A fase pacote/instalação é acionada somente quando o manifesto declara payload instalável.

## Razão

Isso cumpre os Aditivos 03 e 05 sem fabricar pacotes vazios. A Etapa 01 atual entrega bibliotecas e contratos, não um executável/serviço de runtime; por isso `NEXXUS_INSTALLABLE=0` e os estados `EMPACOTADO`/`INSTALADO` são N/A, não simulados.

## Consequência

A primeira etapa futura que introduzir payload instalável deverá implementar/ativar o driver nativo Arch e Debian consumindo este contrato comum, sem criar infraestrutura paralela.
