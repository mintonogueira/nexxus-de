# HANDOFF FINAL — NEXXUS — ETAPA 04 — BACKEND X11

**Projeto:** Nexxus  
**Etapa:** 04 — Backend X11  
**Módulo:** `nexxus-backend-x11`  
**Versão:** 0.1.0  
**Status:** VALIDADO / PUBLICADO / ENTREGUE  
**Data-base:** 2026-08-16

## 1. Repositório e rastreabilidade

- **REPOSITORIO_GITHUB:** `https://github.com/mintonogueira/nexxus-de`
- **BRANCH_CANONICA:** `main`
- **PASTA_DA_ETAPA:** `etapas/etapa-04-backend-x11/`
- **BRANCH_DE_IMPLEMENTACAO:** `etapa-04-backend-x11-impl`
- **PR_DE_PUBLICACAO:** `#3`
- **COMMIT_MAIN_VALIDADO:** `081b152e863f5b91dafd481e864f7fa8658f530a`
- **COMMIT_TECNICO_BRANCH:** `100ed8e6ccf43a6bb162dc5224fce275ba53d29b`
- **COMMIT_ARTEFATOS_BRANCH:** `0023ee44fa7cbc42b90c8df2cb3e4264a099d839`
- **WORKFLOW_MAIN:** `31985046800` — SUCCESS
- **WORKFLOW_TECNICO_BRANCH:** `31984901974` — SUCCESS
- **STATUS_GITHUB:** PUBLICADO
- **PENDENCIAS_DE_PUBLICACAO:** nenhuma

`COMMIT_MAIN_VALIDADO` identifica o estado técnico publicado na branch canônica e revalidado com sucesso nos cenários Arch Linux e Debian. Este handoff é documentação de encerramento posterior e não modifica a implementação validada.

## 2. Resultado entregue

A Etapa 04 entrega o primeiro backend gráfico concreto do Nexxus. O backend traduz eventos e propriedades X11 em contratos abstratos do Window Manager Core e executa comandos abstratos do WM no X server, sem duplicar a lógica da Etapa 02.

Implementado:

- crate `nexxus-backend-x11` em Rust;
- conexão X11 por `x11rb 0.14.0` com `RustConnection`;
- aquisição verificada de `SubstructureRedirect` na root window;
- erro explícito quando outro window manager já controla a sessão X11;
- integração ao `nexxus-backend-api` da Etapa 01;
- módulo `NexxusModule` com ID canônico `nexxus-backend-x11`;
- capability `graphics.backend` consumível pelo Session Runtime da Etapa 03;
- integração direta ao `WindowManager` da Etapa 02;
- scan de janelas X11 já existentes no startup;
- gerenciamento de `MapRequest`, `DestroyNotify`, `UnmapNotify`, `ConfigureRequest`, `ConfigureNotify`, `FocusIn`, `PropertyNotify` e `ClientMessage` pertinentes;
- toplevels, foco, raise, move, resize, maximize, restore, fullscreen e close;
- preservação das restrições de tamanho ICCCM por `WM_NORMAL_HINTS`;
- metadados por `_NET_WM_NAME`, `WM_NAME` e `WM_CLASS`;
- fechamento cooperativo via `WM_DELETE_WINDOW`, com fallback de protocolo quando necessário;
- `WM_TAKE_FOCUS` quando anunciado pelo cliente;
- publicação de `_NET_SUPPORTED`, `_NET_SUPPORTING_WM_CHECK`, `_NET_CLIENT_LIST`, `_NET_CLIENT_LIST_STACKING` e `_NET_ACTIVE_WINDOW`;
- tratamento de `_NET_ACTIVE_WINDOW`, `_NET_CLOSE_WINDOW` e `_NET_WM_STATE` para maximize/fullscreen;
- worker X11 dedicado com ordenação determinística entre eventos e comandos;
- controller backend-neutral para testes/integração;
- binário de smoke test `nexxus-x11-backend-check`;
- scripts Shell 100% POSIX separados para Arch Linux e Debian;
- empacotamento e instalação nativos dos dois cenários;
- snapshot `.tar.gz` e SHA-256.

## 3. Contratos e arquitetura final

### 3.1 Fronteira X11 ↔ Window Manager Core

O X11 é confinado ao crate desta etapa. IDs X11 são normalizados para `WindowId` e eventos do servidor são convertidos para `WmEvent`. Operações solicitadas pelo núcleo continuam usando `WmCommand` e o backend converte somente a execução física dessas operações para X11.

O `WindowManager` permanece a fonte da lógica comum de estado, foco, geometria e restauração; o backend não cria um segundo motor de gerenciamento de janelas.

### 3.2 Session Runtime

`X11BackendModule` cumpre o contrato deixado pela Etapa 03:

- `BackendKind::X11`;
- módulo `nexxus-backend-x11`;
- capability `graphics.backend`;
- lifecycle `initialize/start/stop`;
- preflight de DISPLAY não invasivo em `initialize`;
- aquisição efetiva do papel de WM somente em `start`, permitindo rollback correto pelo lifecycle comum.

### 3.3 EWMH/ICCCM

Foi implementado o subconjunto pertinente ao escopo atual para identidade do WM, clientes gerenciados, janela ativa, estados maximize/fullscreen, fechamento e foco cooperativo.

Protocolos de workspace não foram antecipados. Eles pertencem à Etapa 05 e deverão estender a integração X11 mediante contrato daquela etapa, sem reescrever silenciosamente este backend.

### 3.4 CSD/SSD e compositor

Nenhuma janela é reparentada e nenhuma decoração Nexxus é desenhada nesta etapa. Assim, CSD de aplicações externas permanece intacto e não existe decoração duplicada introduzida pelo Backend X11. Window Chrome/SSD próprio pertence à etapa específica posterior.

O compositor X11 não foi ativado porque não é tecnicamente necessário para cumprir foco, geometria, lifecycle, EWMH ou ICCCM desta etapa. Nenhum blur, fade, sombra decorativa, transparência ou animação foi introduzido.

## 4. Decisão técnica relevante

`docs/ADR-001-x11rb-and-compositor-policy.md` registra:

- adoção de `x11rb 0.14.0`;
- uso de conexão Rust pura;
- `default-features = false`;
- não habilitação de `allow-unsafe-code`/XCB FFI;
- manutenção de `#![forbid(unsafe_code)]` no crate;
- ausência deliberada de compositor quando não requerido;
- ausência deliberada de reparenting/decoração nesta etapa.

A decisão reduz FFI/unsafe, dependências nativas, footprint e complexidade sem alterar o resultado funcional aprovado.

## 5. Testes e validações executados

A validação técnica da branch foi executada no workflow `31984901974`, com SUCCESS em Arch Linux, Debian e delivery. Após o merge, a mesma implementação foi reexecutada na branch `main` pelo workflow `31985046800`, com SUCCESS nos jobs Arch Linux e Debian; o job de delivery é intencionalmente restrito à branch de implementação e foi SKIPPED na `main`.

Em ambos os cenários pertinentes foram executados:

- auditoria POSIX dos wrappers;
- `cargo build --workspace --release`;
- `cargo fmt --all -- --check`;
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`;
- `cargo test --workspace --all-features`;
- `cargo doc --workspace --all-features --no-deps` com warnings tratados como erro;
- Xvfb como X server isolado para os testes reais;
- staging isolado;
- geração e validação do pacote nativo;
- instalação exatamente do pacote gerado;
- smoke test do binário instalado sob Xvfb;
- validação do snapshot de entrega.

### 5.1 Cobertura funcional X11 real

O harness cria uma janela X11 real sob Xvfb e valida:

- MapRequest e registro da janela;
- cliente permanece filho direto da root window, provando ausência de reparenting desta etapa;
- `_NET_SUPPORTING_WM_CHECK` publicado;
- `_NET_CLIENT_LIST` contém a janela gerenciada;
- foco;
- movimento;
- redimensionamento;
- maximize → restore;
- fullscreen → restore;
- recebimento real de `WM_DELETE_WINDOW` pelo cliente;
- destruição do cliente e remoção determinística do estado do WM;
- integração do módulo X11 concreto com `SessionRuntime`.

### 5.2 Smoke test pós-instalação

O binário instalado a partir dos pacotes nativos foi executado sob Xvfb e confirmou:

- `backend=x11`;
- `output=1280x720`;
- `wm_claim=ok`;
- `compositor=not-required`.

## 6. Build e pacotes nativos

### Arch Linux

- **PACOTE:** `dist/arch/nexxus-backend-x11-0.1.0-1-x86_64.pkg.tar.zst`
- **SHA-256:** `780b9f68044d74a9227ad3f33bc1dc1d81f4de4bbac6a5354de38e67ec611a29`
- **STATUS:** GERADO / VALIDADO / INSTALADO / TESTADO

`scripts/build-install-arch.sh` é Shell POSIX, autoprovisiona dependências ausentes pelo `pacman`, compila/testa como usuário normal, faz staging, gera pacote nativo via `makepkg`, valida seu conteúdo, instala exatamente o pacote produzido via `pacman -U` e executa smoke test do binário instalado.

### Debian

- **PACOTE:** `dist/debian/nexxus-backend-x11_0.1.0_amd64.deb`
- **SHA-256:** `22a8b9a3dfbdd0618851cb14a8a5510bf2c9afee692157f7992779881bf9e716`
- **STATUS:** GERADO / VALIDADO / INSTALADO / TESTADO

`scripts/build-install-debian.sh` é Shell POSIX, autoprovisiona dependências ausentes pelo APT, compila/testa como usuário normal, faz staging, gera `.deb`, valida seu conteúdo, instala exatamente o pacote produzido via APT e executa smoke test do binário instalado.

## 7. Entrega compactada

- **ARQUIVO:** `Nexxus_Etapa04_Backend_X11_0.1.0.tar.gz`
- **PATH:** `etapas/etapa-04-backend-x11/entrega/Nexxus_Etapa04_Backend_X11_0.1.0.tar.gz`
- **SHA-256:** `f3548511e63348d4ade7590dc07d56f70a978d19d9c595be43376dee27c1b102`
- **STATUS:** VALIDADA / VERSIONADA / ENTREGUE

O snapshot contém código da etapa, `Cargo.lock`, documentação, ADR, manifests, wrappers POSIX, packaging e os pacotes nativos validados. Caches, `target/`, staging temporário e segredos não fazem parte da entrega.

## 8. Dependências consumidas

Das etapas anteriores:

- `nexxus-core`;
- `nexxus-backend-api`;
- `nexxus-wm`;
- `nexxus-session` apenas no harness/contrato de integração.

Dependências Rust diretas da etapa:

- `x11rb 0.14.0`;
- `thiserror`.

Nenhum GTK, Qt, Electron, JVM ou Python foi introduzido no componente.

## 9. Limitações intencionais / fora do escopo

Não são defeitos pendentes da Etapa 04:

- Workspace Manager;
- Tiling Engine;
- UI final;
- Window Chrome/decoração Nexxus;
- atalhos globais;
- backend Wayland/XWayland;
- XDG Desktop Portals;
- compositor com efeitos;
- política completa de multi-monitor/hotplug.

Os hooks e contratos necessários para evolução posterior foram preservados sem antecipar implementação de módulos futuros.

## 10. Critérios de aceite

- backend X11 concreto funcional: **ATENDIDO**;
- aplicações X11 reais são descobertas/mapeadas e gerenciadas no harness: **ATENDIDO**;
- foco: **ATENDIDO**;
- move/resize: **ATENDIDO**;
- fechamento cooperativo ICCCM: **ATENDIDO**;
- maximize/restore: **ATENDIDO**;
- fullscreen/restore: **ATENDIDO**;
- EWMH/ICCCM pertinentes: **ATENDIDO**;
- CSD/SSD sem reparenting/decoração duplicada introduzida pela etapa: **ATENDIDO**;
- ausência de efeitos visuais proibidos: **ATENDIDO**;
- integração com WM Core e Session Runtime: **ATENDIDO**;
- `#![forbid(unsafe_code)]`: **ATENDIDO**;
- release build/rustfmt/Clippy/testes/rustdoc: **ATENDIDO**;
- wrappers POSIX Arch/Debian: **ATENDIDO**;
- pacotes nativos gerados, validados, instalados e testados: **ATENDIDO**;
- snapshot e SHA-256: **ATENDIDO**;
- publicação e revalidação na `main`: **ATENDIDO**.

## 11. Pendências

Nenhuma pendência bloqueante pertence à Etapa 04.

Funcionalidades deliberadamente posteriores permanecem nas etapas próprias conforme o Plano Mestre, sem invasão de escopo.

## 12. Próxima etapa

**ETAPA ATUAL:** 04 — Backend X11  
**STATUS:** VALIDADO / PUBLICADO / ENTREGUE  
**PRÓXIMA ETAPA RECOMENDADA:** Etapa 05 — Workspace Manager  
**NOVA CONVERSA:** `NEXXUS - Etapa 05 - Workspace Manager`  
**OBJETIVO:** implementar workspaces fixas e dinâmicas como contexto coerente distribuível por todos os monitores, sem vínculo rígido workspace-monitor.  
**DEPENDÊNCIAS DISPONÍVEIS:** Etapas 01, 02, 03 e 04 validadas.  

A Etapa 05 deve ser desenvolvida exclusivamente em nova conversa. Este handoff não inicia sua implementação.
