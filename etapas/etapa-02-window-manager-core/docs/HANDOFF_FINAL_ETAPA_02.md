# Handoff Final — Etapa 02 — Window Manager Core

## ETAPA ATUAL

Etapa 02 — Window Manager Core

## MÓDULO

`nexxus-wm`

## STATUS

**VALIDADA — versão 0.1.0, pronta para integração/publicação na `main`.**

## ENTREGÁVEIS PRODUZIDOS

- workspace Rust da Etapa 02 e crate `nexxus-wm`;
- identificador interno `WindowId` sem dependência de handle nativo;
- modelo de geometria e `SizeConstraints` com validação defensiva;
- modelo lógico de janelas e metadados normalizados;
- lifecycle lógico de criação, map/unmap, alteração e destruição;
- foco ativo, MRU e recuperação determinística de foco;
- `WindowPlacement` (`Floating`/`Tiled`) separado de `PresentationState` (`Normal`/`Maximized`/`Fullscreen`);
- restauração determinística por `RestoreSnapshot` e preservação da geometria floating;
- eventos `WmEvent`, comandos `WmCommand` e trait `BackendCommandSink`;
- tratamento explícito de eventos stale após destruição;
- testes unitários e de contrato;
- ADR do modelo de estado/restore e documentação dos contratos;
- wrappers Shell 100% POSIX separados para Arch Linux e Debian;
- manifesto operacional, auditoria POSIX e auditoria de neutralidade de backend;
- workflow GitHub Actions dedicado;
- snapshot portátil e arquivo SHA-256.

## DEPENDÊNCIAS

Consumidas da Etapa 01, sem reconstrução:

- `nexxus-core`;
- `nexxus-backend-api`.

Dependência Rust adicional utilizada: `thiserror`, já compatível com o padrão de tratamento de erros da fundação. `nexxus-protocol` e `nexxus-config` não foram adicionados porque a Etapa 02 não possui necessidade própria de IPC ou persistência.

## CONTRATOS INTERMODULARES

### Entrada

`WmEvent` recebe eventos normalizados de futuros adaptadores/backends gráficos.

### Saída

`WmCommand` expressa intenção lógica de foco, move, resize, maximize, restore, fullscreen e close. `BackendCommandSink` constitui a fronteira mínima para um backend futuro executar essas intenções.

### Regra de isolamento

Nenhum handle, tipo ou dependência concreta de X11, Wayland, XWayland, Smithay, XCB ou wlroots integra o contrato público do `nexxus-wm`.

## TESTES E VALIDAÇÕES EXECUTADOS

GitHub Actions run final da revisão técnica: `31980820527`.

- commit técnico validado: `9cbc6d9fde1f40f0a9b49b525ced8e798f156d95`;
- Arch Linux current: success;
- Debian Trixie: success;
- snapshot-entrega: success;
- `cargo build --workspace --release`: aprovado;
- `cargo fmt`: aprovado e fontes normalizadas;
- Clippy com `-D warnings`: aprovado;
- `cargo test --workspace --all-features`: aprovado;
- rustdoc com warnings como erro: aprovado;
- auditoria Shell POSIX: aprovada;
- auditoria do grafo para ausência de backend gráfico concreto: aprovada;
- `Cargo.lock`: gerado pelo Cargo e versionado.

## DISTRIBUIÇÃO / EMPACOTAMENTO

- REVISADO: SIM;
- COMPILADO: SIM;
- TESTADO: SIM;
- EMPACOTADO NATIVO: N/A;
- INSTALADO: N/A;
- VALIDADO: SIM.

A Etapa 02 produz biblioteca interna e declara `NEXXUS_INSTALLABLE=0`. Não existe payload runtime instalável; portanto, conforme a governança já aplicada na fundação, nenhum pacote Arch/Debian vazio foi fabricado e nenhuma instalação artificial foi executada.

## ENTREGA COMPACTADA

- arquivo: `Nexxus_Etapa02_Window_Manager_Core_0.1.0.tar.gz`;
- path versionado: `etapas/etapa-02-window-manager-core/entrega/Nexxus_Etapa02_Window_Manager_Core_0.1.0.tar.gz`;
- SHA-256: `26c39a1f87e2d0bc5fe6f1bfa9dd77437b354fe7f62eb59e70224889644ed9e1`;
- run que gerou/validou o snapshot: `31980820527`;
- commit que adiciona o snapshot e corrige os modos executáveis dos scripts: `ab51cd12c5663fd29d9131b8f62dc7338f627d41`.

## DECISÕES TÉCNICAS RELEVANTES

1. `BTreeMap` foi usado no registry de janelas para iteração determinística; `VecDeque` mantém a ordem MRU.
2. Placement e presentation são eixos independentes para evitar enum combinatório e preservar restauração correta.
3. Move/resize são pedidos abstratos; a geometria confirmada é atualizada pelo evento retornado do backend.
4. Eventos atrasados para janelas já destruídas são classificados como stale e não recriam estado.
5. O estado lógico de tiling existe, mas cálculo de layout/slot permanece fora desta etapa e pertence ao futuro Tiling Engine.
6. `#![forbid(unsafe_code)]` permanece ativo no crate.
7. Nenhum runtime assíncrono foi introduzido por não ser necessário ao núcleo lógico atual.

## SEGURANÇA E LIMITES

- nenhum `unsafe` no crate;
- nenhuma execução de build como root;
- elevação dos wrappers limitada ao gerenciador de pacotes para dependências ausentes;
- limpeza de staging limitada a paths validados;
- nenhuma credencial/segredo no código ou logs;
- nenhuma implementação concreta de protocolo gráfico nesta etapa.

## LIMITAÇÕES INTENCIONAIS

O Window Manager Core ainda não move ou apresenta janelas reais na tela. Isso é intencional: execução física depende de backends gráficos posteriores. Workspaces e algoritmos de tiling também permanecem fora desta etapa conforme a divisão oficial do Nexxus.

## GITHUB

- repositório canônico: `https://github.com/mintonogueira/nexxus-de`;
- branch da etapa: `etapa-02-window-manager-core`;
- pasta: `etapas/etapa-02-window-manager-core/`;
- commit técnico validado: `9cbc6d9fde1f40f0a9b49b525ced8e798f156d95`;
- commit de snapshot/modos executáveis: `ab51cd12c5663fd29d9131b8f62dc7338f627d41`;
- STATUS_GITHUB neste ponto do handoff: `VALIDADO_NA_BRANCH / PRONTO_PARA_MAIN`.

## PENDÊNCIAS DA ETAPA 02

Nenhuma pendência funcional conhecida dentro do escopo da Etapa 02. Resta somente a operação de governança de integrar esta entrega à `main` e registrar o commit/PR resultante.

## PRÓXIMA ETAPA RECOMENDADA

**Etapa 03 — Session Runtime.**

A Etapa 03 deverá consumir a fundação da Etapa 01 e os contratos validados do Window Manager Core, sem antecipar Backend X11, Workspace Manager, Tiling Engine ou outros módulos posteriores.

## NOVA CONVERSA

`NEXXUS - Etapa 03 - Session Runtime`

A Etapa 03 não deve ser iniciada nesta conversa.
