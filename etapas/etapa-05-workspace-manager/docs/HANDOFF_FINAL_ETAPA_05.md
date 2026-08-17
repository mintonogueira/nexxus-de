# HANDOFF FINAL — NEXXUS — ETAPA 05 — WORKSPACE MANAGER

**Projeto:** Nexxus  
**Etapa:** 05 — Workspace Manager  
**Módulo:** `nexxus-workspaces`  
**Versão:** 0.1.0  
**Status:** VALIDADO / PUBLICADO / ENTREGUE  
**Data-base:** 2026-08-16

## 1. Repositório e rastreabilidade

- **REPOSITORIO_GITHUB:** `https://github.com/mintonogueira/nexxus-de`
- **BRANCH_CANONICA:** `main`
- **PASTA_DA_ETAPA:** `etapas/etapa-05-workspace-manager/`
- **BRANCH_DE_IMPLEMENTACAO:** `etapa-05-workspace-manager-impl`
- **PR_DE_PUBLICACAO:** `#4`
- **COMMIT_BRANCH_VALIDADO:** `cfadfc42e85f0fafbc9665399d439771a13ff3d8`
- **COMMIT_MAIN_VALIDADO:** `cd2e0d32445076f54701f9b724a3532eda3fd01f`
- **WORKFLOW_BRANCH:** `31985950552` — SUCCESS
- **WORKFLOW_MAIN:** `31986088565` — SUCCESS nos cenários Arch Linux e Debian
- **STATUS_GITHUB:** PUBLICADO
- **PENDENCIAS_DE_PUBLICACAO:** nenhuma referente à implementação

O workflow de delivery é executado apenas na branch de implementação. Na `main`, ele é intencionalmente `SKIPPED`; a revalidação da implementação continua sendo feita nos dois cenários de distribuição.

## 2. Resultado entregue

A Etapa 05 entrega o núcleo lógico backend-neutral de workspaces do Nexxus. O módulo modela workspaces como contexto de trabalho independente de monitor e preserva a regra normativa de que uma mesma workspace pode distribuir suas janelas por todos os monitores.

Implementado:

- crate Rust `nexxus-workspaces`;
- `WorkspaceId` persistente e não zero, inclusive na desserialização;
- workspaces `Fixed` e `Dynamic`;
- workspace ativa;
- ordem estável e histórico/MRU determinístico;
- membership `WindowId -> WorkspaceId`;
- criação, remoção, rename e ativação;
- movimentação manual de janelas entre workspaces;
- regras de aplicação para placement inicial apenas;
- garantia de que placement inicial nunca aprisiona a janela;
- remoção de workspace com realocação determinística das janelas antes da exclusão;
- política de workspaces dinâmicas `keep-empty` ou `remove-empty-inactive`;
- eventos `Created`, `Removed`, `Renamed`, `Activated`, `WindowMoved` e `WindowForgotten`;
- exposição das janelas da workspace ativa para futura filtragem do Alt+Tab;
- MRU de workspaces para consumo futuro do Super+Tab;
- persistência TOML versionada via `nexxus-config::TomlConfigStore`;
- auditoria explícita de neutralidade contra dependências concretas X11/Wayland;
- scripts Shell 100% POSIX separados para Arch Linux e Debian;
- snapshot versionável e SHA-256.

## 3. Arquitetura final

### 3.1 Modelo de workspace

A workspace não contém identificador de monitor. Ela possui identidade, nome, tipo e conjunto de `WindowId`. A distribuição física das janelas continua sendo representada pelas geometrias administradas pelo Window Manager Core.

Isso preserva a regra:

`workspace = contexto lógico`  
`monitor = destino físico de geometria`  

Não existe binding rígido `workspace -> monitor`.

### 3.2 Integração com o Window Manager Core

O Workspace Manager consome `nexxus-wm::WindowId` como identidade backend-neutral. Ele não recria foco, geometria, maximize, fullscreen ou lifecycle de janelas; essas responsabilidades permanecem no `nexxus-wm`.

Para o futuro Alt+Tab, o Workspace Manager fornece o conjunto de janelas pertencentes à workspace atual; a ordenação de foco das janelas permanece responsabilidade do MRU do WM.

### 3.3 Placement inicial sem aprisionamento

`PlacementRule` relaciona `application_id` a uma workspace e é avaliada somente na entrada `assign_new_window`. Depois da associação inicial, `move_window` não reavalia nem reaplica a regra.

Assim, uma aplicação pode abrir automaticamente em uma workspace definida e ainda ser movida livremente pelo usuário para qualquer outra workspace.

### 3.4 Remoção sem perda de janelas

Ao remover uma workspace ocupada, o módulo determina primeiro uma workspace sobrevivente, realoca todas as janelas residentes, atualiza membership e eventos e somente então remove a workspace antiga.

Quando a workspace removida era a ativa, a nova ativação também é registrada de forma determinística.

### 3.5 Workspaces dinâmicas

O lifecycle automático é configurável por `DynamicPolicy`:

- `KeepEmpty` — preserva workspaces dinâmicas vazias;
- `RemoveEmptyInactive` — remove uma workspace dinâmica quando ela fica vazia e não está ativa.

A criação continua sendo operação explícita do manager. Interfaces e Settings posteriores poderão escolher quando solicitar a criação sem alterar o núcleo.

### 3.6 Persistência e Session State

A configuração persistente armazena:

- workspace ativa;
- política dinâmica;
- definições de workspaces;
- regras de placement.

Membership de janelas/processos em execução não é persistido nesta configuração. A restauração completa de sessão pertence à Etapa 53 — Session State e não foi antecipada.

## 4. Contratos públicos da etapa

Tipos principais:

- `WorkspaceId`;
- `WorkspaceKind`;
- `DynamicPolicy`;
- `WorkspaceDefinition`;
- `PlacementRule`;
- `WorkspaceConfig`;
- `Workspace`;
- `WorkspaceEvent`;
- `WorkspaceManager`;
- `WorkspaceError`.

Operações relevantes:

- `with_single_fixed`;
- `from_config` / `load` / `save`;
- `create_fixed` / `create_dynamic`;
- `rename`;
- `activate`;
- `assign_new_window`;
- `move_window`;
- `forget_window`;
- `active_windows`;
- `mru_order` / `previous_mru`;
- `drain_events`;
- `config_snapshot`.

O contrato não expõe X11, Wayland, XCB, Smithay, DRM ou IDs de monitor.

## 5. Decisão técnica relevante

`docs/ADR-001-workspace-model-and-persistence.md` registra:

- workspace como contexto lógico sem monitor embutido;
- placement somente no primeiro assignment;
- remoção com realocação antes da exclusão;
- política dinâmica configurável;
- persistência pelo `nexxus-config` da fundação;
- não persistência de membership runtime;
- dependência do `WindowId` backend-neutral;
- futura adaptação gráfica consumindo eventos/estado, sem transferir lógica do Workspace Manager ao backend.

## 6. Testes e validações executados

A execução de validação da branch `31985950552` terminou com **SUCCESS** em:

- Arch Linux current;
- Debian Trixie;
- delivery.

Após merge do PR #4, a `main` foi revalidada pelo workflow `31986088565` com **SUCCESS** em Arch Linux e Debian.

Foram executados:

- auditoria POSIX dos wrappers;
- `cargo build --workspace --release`;
- `cargo fmt --all -- --check`;
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`;
- `cargo test --workspace --all-features`;
- `cargo doc --workspace --all-features --no-deps` com warnings tratados como erro;
- auditoria contra dependências/APIs concretas de backend;
- staging isolado.

### 6.1 Cobertura funcional

Os testes validam:

- criação de workspaces fixas e dinâmicas;
- ativação e MRU determinísticos;
- placement inicial por regra;
- movimentação manual posterior ignorando a regra inicial;
- remoção automática de workspace dinâmica vazia/inativa;
- movimentação de janela para workspace sobrevivente sem perda;
- remoção da workspace ativa sem perder suas janelas;
- roundtrip de configuração persistente;
- ausência intencional de restauração de membership runtime pelo arquivo de configuração.

## 7. Incidentes de validação e correções

A CI encontrou e bloqueou três problemas durante o desenvolvimento, todos corrigidos antes da validação final:

1. auditoria POSIX examinava seu próprio padrão de detecção e produzia falso positivo;
2. expressão regular literal do verificador de neutralidade era confundida com sintaxe Bash pela auditoria textual;
3. Clippy atual identificou implementação manual de `Default` que podia ser derivada.

As correções não relaxaram os critérios. A auditoria POSIX passou a ignorar strings literais no exame lexical; `DynamicPolicy` passou a usar `derive(Default)` e o contrato de `WorkspaceId` foi adicionalmente endurecido contra desserialização de zero.

## 8. Build, packaging e instalação

Foram mantidos os dois pontos de entrada obrigatórios:

- `scripts/build-install-arch.sh`;
- `scripts/build-install-debian.sh`.

Ambos são Shell 100% POSIX, validam a distribuição, autoprovisionam dependências ausentes e executam build/testes/staging como usuário normal.

`nexxus-workspaces` é biblioteca interna nesta etapa e não possui payload runtime independente. Portanto:

- **NEXXUS_INSTALLABLE:** `0`;
- **PACOTE ARCH:** N/A;
- **PACOTE DEBIAN:** N/A;
- **INSTALAÇÃO:** N/A.

Nenhum pacote vazio foi fabricado apenas para satisfazer formalmente o pipeline.

## 9. Entrega compactada

- **ARQUIVO:** `Nexxus_Etapa05_Workspace_Manager_0.1.0.tar.gz`
- **PATH:** `etapas/etapa-05-workspace-manager/entrega/Nexxus_Etapa05_Workspace_Manager_0.1.0.tar.gz`
- **SHA-256:** `9923ed76531f613c02ff676665e863575924e6306ac51e140b3d962cc255bb4e`
- **STATUS:** VALIDADA / VERSIONADA / ENTREGUE

O snapshot contém código, `Cargo.lock`, documentação, ADR, manifests e wrappers pertencentes à etapa; caches e staging temporário são excluídos.

## 10. Dependências consumidas

Das etapas anteriores:

- `nexxus-config`;
- `nexxus-wm`;
- dependências transitivas pertencentes às etapas anteriores.

Dependências Rust diretas novas do crate:

- `serde`;
- `thiserror`.

Nenhum GTK, Qt, Electron, JVM, Python, X11 ou Wayland foi introduzido no núcleo lógico da etapa.

## 11. Critérios de aceite

- sem binding rígido workspace-monitor: **ATENDIDO**;
- workspaces fixas nomeáveis: **ATENDIDO**;
- workspaces dinâmicas e política de remoção: **ATENDIDO**;
- workspace atual e histórico/MRU: **ATENDIDO**;
- movimentação de janelas: **ATENDIDO**;
- placement inicial não impede movimento manual: **ATENDIDO**;
- criação/remoção dinâmica não perde janelas: **ATENDIDO**;
- eventos create/remove/rename/activate/move-window: **ATENDIDO**;
- base para Alt+Tab da workspace atual: **ATENDIDO**;
- base para Super+Tab por histórico: **ATENDIDO**;
- persistência da configuração: **ATENDIDO**;
- contratos backend-neutral: **ATENDIDO**;
- build release/rustfmt/Clippy/testes/rustdoc: **ATENDIDO**;
- Shell POSIX Arch/Debian: **ATENDIDO**;
- snapshot + SHA-256: **ATENDIDO**;
- publicação e revalidação na `main`: **ATENDIDO**.

## 12. Limitações intencionais / fora do escopo

Não são pendências da Etapa 05:

- Tiling Engine e layouts por workspace;
- Snap Layouts e Super+T;
- Workspace Bar visual;
- editor gráfico de regras;
- Settings de workspaces;
- Session State completo;
- backend Wayland/XWayland;
- implementação visual de Alt+Tab/Super+Tab;
- topologia e hotplug de monitores.

## 13. Pendências

Nenhuma pendência bloqueante pertence à Etapa 05.

## 14. Próxima etapa

**ETAPA ATUAL:** 05 — Workspace Manager  
**STATUS:** VALIDADO / PUBLICADO / ENTREGUE  
**PRÓXIMA ETAPA RECOMENDADA:** Etapa 06 — Tiling Engine  
**NOVA CONVERSA:** `NEXXUS - Etapa 06 - Tiling Engine`  
**OBJETIVO:** implementar tiling assistido e snap visual como organizador opcional, preservando janelas livres e a geometria floating.  
**DEPENDÊNCIAS DISPONÍVEIS:** Etapas 01–05 validadas.  

A Etapa 06 deve ser desenvolvida exclusivamente em nova conversa. Este handoff encerra a Etapa 05 e não inicia sua implementação.
