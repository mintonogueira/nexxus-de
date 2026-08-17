# HANDOFF FINAL — NEXXUS ETAPA 06 — TILING ENGINE

## Identificação

- **Projeto:** Nexxus
- **Etapa:** 06 — Tiling Engine
- **Módulo:** `nexxus-tiling`
- **Versão:** 0.1.0
- **Status técnico:** VALIDADO
- **Status de entrega:** ENTREGUE / PUBLICADO
- **Repositório canônico:** `https://github.com/mintonogueira/nexxus-de`
- **Branch canônica:** `main`
- **PR de implementação:** #6
- **Commit publicado em `main`:** `687efab9c3adf7a3a8d067c390fd374969625438`

## Resultado da etapa

A Etapa 06 entrega o motor backend-neutral de tiling assistido do Nexxus. O módulo organiza janelas por layouts e snap sem transformar o tiling em estado obrigatório: a janela pode ser encaixada pontualmente e retorna ao comportamento floating quando liberada para movimentação/redimensionamento manual.

O módulo preserva a separação arquitetural estabelecida pelo projeto:

- `nexxus-wm` continua proprietário do estado da janela, constraints e geometria floating;
- `nexxus-workspaces` continua proprietário de workspaces e membership de janelas;
- `nexxus-tiling` calcula layout, slot, snap e planos geométricos;
- backends recebem somente `WmCommand` pelo contrato `BackendCommandSink` existente;
- nenhuma dependência concreta X11/Wayland foi incorporada ao engine;
- workspace permanece contexto lógico e não é vinculada rigidamente a monitor.

## Funcionalidades implementadas

- layouts independentes por `WorkspaceId`;
- fallback interno balanceado em duas colunas enquanto a UI de Settings ainda não existe;
- slots normalizados em fixed-point de 10.000 unidades;
- cálculo determinístico para áreas úteis e múltiplos outputs;
- suporte a coordenadas globais negativas para monitores posicionados à esquerda da origem;
- arredondamento cumulativo ao pixel físico mais próximo, preservando limites compartilhados e evitando gaps;
- alocação determinística de slots por `(workspace, output)`;
- ação estável `nexxus.tiling.tile-fit`;
- descriptor do atalho aprovado `Super+T` sem antecipar o Shortcuts Core;
- snap direto em metade esquerda/direita e quatro quadrantes de canto;
- topo central gera `ShowLayoutChoices` para o futuro overlay gráfico;
- `untile` restaura a geometria floating preservada pelo Window Manager Core;
- `release_for_manual_operation` devolve automaticamente a liberdade floating antes de move/resize manual;
- constraints `min/max` aplicadas dentro do slot;
- mínimo que não cabe gera erro antes de qualquer mutação de placement;
- máximo menor que o slot centraliza a janela dentro do slot;
- eventos de mudança de layout, tiling, release e solicitação de layout chooser;
- limpeza de assignments quando a janela deixa de pertencer ao contexto de tiling.

## API/contratos principais

### Ações e constantes

- `TILE_FIT_ACTION_ID = "nexxus.tiling.tile-fit"`
- `DEFAULT_TILE_FIT_SHORTCUT = "Super+T"`
- `TilingAction::TileFit`

### Geometria/layout

- `NormalizedRect`
- `LayoutSpec`
- `OutputId`
- `OutputArea`
- `LayoutError`

### Engine

- `TilingEngine::set_layout`
- `TilingEngine::clear_layout`
- `TilingEngine::tile_fit_active`
- `TilingEngine::tile_fit`
- `TilingEngine::apply_snap`
- `TilingEngine::untile`
- `TilingEngine::release_for_manual_operation`
- `TilingEngine::dispatch_tile_plan`
- `TilingEngine::dispatch_untile_plan`

### Snap

- `SnapDetector`
- `SnapIntent`
- `SnapTarget`
- `Point`

## Dependências consumidas

- **Etapa 02 — Window Manager Core:** `Geometry`, `SizeConstraints`, `WindowPlacement`, `PresentationState`, `WindowManager`, `WmCommand`, `BackendCommandSink`.
- **Etapa 05 — Workspace Manager:** `WorkspaceId`, `WorkspaceManager`, workspace ativa e membership de janelas.
- **Etapa 04 — Backend X11:** consumidor inicial indireto dos `WmCommand`; nenhuma implementação X11 foi movida para esta etapa.

Nenhum módulo de outra etapa foi implementado ou refatorado dentro da Etapa 06.

## Decisão técnica registrada

`docs/ADR-001-assisted-tiling-geometry.md` registra a arquitetura interna do solver, fixed-point, relação workspace/output, restauração floating, política de constraints, snap e fronteira com o futuro Shortcuts Core/UI.

A ADR é decisão interna de implementação e **não altera requisito normativo do Nexxus**. Portanto, não foi necessária nova documentação aditiva normativa.

## Testes e validações

### Rust

- `cargo build --workspace --release`: aprovado;
- `cargo fmt --package nexxus-tiling -- --check`: aprovado;
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: aprovado;
- `cargo test --workspace --all-features`: aprovado;
- `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --all-features --no-deps`: aprovado;
- **15 testes Rust aprovados**: 5 unitários + 10 de integração.

Cenários cobertos incluem layouts distintos por workspace, múltiplos outputs, coordenadas negativas, constraints min/max, slot exhaustion, snap, `Super+T`/tile-fit, restauração floating e liberação para move/resize manual.

### Shell e isolamento

- auditoria Shell 100% POSIX: aprovada;
- wrappers separados Arch/Debian: aprovados;
- auditoria de neutralidade de backend: aprovada;
- dependências de etapas anteriores permanecem somente leitura durante a validação.

### CI real

**Workflow da branch de implementação:** `31987742174`

- Arch Linux: SUCCESS;
- Debian Trixie: SUCCESS;
- delivery/snapshot: SUCCESS.

**Workflow pós-merge em `main`:** `31987843352`

- Arch Linux: SUCCESS;
- Debian Trixie: SUCCESS;
- delivery: SKIPPED por design em `main`, pois o artefato já havia sido produzido e versionado na branch validada.

## Incidentes encontrados e corrigidos

### 1. Fronteira de formatação entre etapas

Na primeira execução do CI, `cargo fmt --all` tentou escrever em crates das Etapas 02/05, mantidas corretamente como somente leitura. A validação foi corrigida para executar `rustfmt` exclusivamente sobre `nexxus-tiling`. Isso preserva a regra de isolamento por etapa sem reduzir a validação do módulo corrente.

Também foi removido um import Rust não utilizado detectado nessa execução.

### 2. Divisão fracionária de três colunas

Um teste encontrou resultado de 599 px onde a divisão de 1800 px em três colunas deveria resultar em 600 px. O problema era o truncamento das fronteiras fixed-point. O solver passou a arredondar cada limite cumulativo ao pixel físico mais próximo. Como slots adjacentes compartilham a mesma fronteira calculada, não são introduzidos gaps.

Após as correções, Arch Linux e Debian passaram integralmente.

## Build, staging e empacotamento

Foram entregues:

- `scripts/build-install-arch.sh` — Shell 100% POSIX;
- `scripts/build-install-debian.sh` — Shell 100% POSIX;
- helpers POSIX comuns;
- manifesto `manifests/etapa-06.conf`;
- autoprovisionamento das dependências necessárias;
- build como usuário normal;
- testes e staging isolado;
- falha segura e códigos de saída coerentes.

### Pacote nativo

**N/A nesta etapa.** `nexxus-tiling` é uma biblioteca interna sem payload runtime independente e está marcado como `NEXXUS_INSTALLABLE=0`. Não foi criado pacote vazio/artificial nem instalação direta fora do gerenciador de pacotes. Quando um componente instalável consumir o módulo, o payload correspondente será empacotado na etapa pertinente conforme os contratos de packaging do Nexxus.

## Artefato de entrega

- `entrega/Nexxus_Etapa06_Tiling_Engine_0.1.0.tar.gz`
- SHA-256: `f03bcd4d7b6de0de8d710ccccd821ee9761d7f1fa3fb9804efeae7143ca277d8`
- `Cargo.lock` validado pelo cenário Debian e versionado junto ao snapshot.

## Critérios de aceite

- `tile-fit`/`Super+T` possui contrato estável: **ATENDIDO**;
- tiling não aprisiona a janela: **ATENDIDO**;
- retorno ao floating restaura a geometria preservada: **ATENDIDO**;
- layouts podem diferir entre workspaces: **ATENDIDO**;
- cálculo respeita área útil e múltiplos monitores: **ATENDIDO**;
- workspace não é presa a monitor: **ATENDIDO**;
- constraints min/max são tratadas: **ATENDIDO**;
- geometrias impossíveis falham antes de mutação parcial: **ATENDIDO**;
- snap produz destinos e hook para overlay futuro: **ATENDIDO**;
- engine permanece backend-neutral: **ATENDIDO**;
- Shell POSIX Arch/Debian: **ATENDIDO**;
- validação real Arch/Debian: **ATENDIDO**;
- snapshot + SHA-256: **ATENDIDO**.

## Fora do escopo preservado

Não foram implementados nesta conversa:

- overlay gráfico definitivo do Snap Layout;
- Settings de tiling;
- Shortcuts Core completo;
- Window Chrome;
- backend Wayland específico;
- qualquer componente da Etapa 07 ou posterior.

## Pendências

Nenhuma pendência bloqueante da Etapa 06.

Integrações futuras já previstas deverão consumir os contratos aqui entregues nas respectivas etapas, sem reabrir silenciosamente o escopo deste módulo.

## Próxima etapa

- **ETAPA ATUAL:** 06 — Tiling Engine
- **STATUS:** VALIDADO / ENTREGUE / PUBLICADO
- **PRÓXIMA ETAPA RECOMENDADA:** Etapa 07 — Nexxus UI Core
- **NOVA CONVERSA:** `NEXXUS FASE 07 — Nexxus UI Core`
- **OBJETIVO:** desenvolver a camada gráfica própria e widgets fundamentais do Nexxus conforme as fontes normativas e o plano mestre.
- **DEPENDÊNCIAS DISPONÍVEIS:** Core/Fundação, Window Manager Core, Backend X11, Workspace Manager e Tiling Engine já publicados/validados.

A Etapa 07 deve ser iniciada exclusivamente em nova conversa.
