# HANDOFF FINAL — NEXXUS ETAPA 07 — NEXXUS UI CORE

## Identificação

- **Projeto:** Nexxus
- **Etapa:** 07 — Nexxus UI Core
- **Módulo:** `nexxus-ui`
- **Versão:** 0.1.0
- **Status técnico:** VALIDADO
- **Status de entrega:** ENTREGUE / PUBLICADO
- **Repositório canônico:** `https://github.com/mintonogueira/nexxus-de`
- **Branch canônica:** `main`
- **PR de implementação:** #8
- **Commit publicado da implementação em `main`:** `2dac3b68135d9bf11836a2238c405ee0d513f015`

## Resultado da etapa

A Etapa 07 entrega a camada gráfica própria e reutilizável do Nexxus em Rust. `nexxus-ui` permanece independente de GTK/Qt e não contém tipos concretos X11/Wayland. A arquitetura usa geometria lógica, uma display list backend-neutral e renderer software que produz frame RGBA8; adapters gráficos posteriores podem apresentar esse frame sem transferir regras de protocolo para a UI.

A etapa preserva a separação arquitetural oficial:

- `nexxus-ui` possui primitivas, widgets, layout, tema, input, foco, hit-testing, acessibilidade semântica e renderização;
- Backend X11 continua responsável pela integração concreta X11;
- Backend Wayland continua pertencendo à etapa própria futura;
- Visual Assets permanece Etapa 08;
- Window Chrome permanece Etapa 09;
- nenhum painel, menu, Settings, File Manager ou outro componente posterior foi implementado.

## Funcionalidades implementadas

- geometria lógica/física (`LogicalPoint`, `LogicalSize`, `LogicalRect`, `PhysicalRect`, `PhysicalSize`);
- escala validada e conversão previsível para pixels físicos;
- arredondamento por bordas compartilhadas para evitar gaps em escala fracionária;
- `Constraints` e `Insets` reutilizáveis;
- paleta semântica dark com superfícies estruturais opacas;
- métricas de controles, padding, gaps, bordas e foco;
- Hack como família semântica padrão, com família configurável;
- `DisplayList` ordenada;
- primitivas de clear, fill/stroke rect, clipping, texto, RGBA image e SVG;
- `Renderer` e `TextMeasurer` como contratos abstratos;
- `SoftwareRenderer` backend-neutral com frame RGBA8;
- texto Unicode com shaping/fallback/rasterização;
- SVG via parser/rasterizador próprio da stack Rust adotada;
- widgets: container, label, button, toggle, checkbox, text field, list, scroll, menu, popup, tabs e spacer;
- foco determinístico e navegação por Tab/Shift+Tab;
- mouse, teclado, scroll e text input normalizados;
- hit-testing pelo topo da ordem de pintura;
- mensagens semânticas de UI em vez de eventos de protocolo gráfico;
- edição UTF-8 preservando fronteiras de caracteres;
- árvore semântica preparada para futuro bridge de acessibilidade;
- harness `nexxus-ui-demo` para inspeção de widgets e renderer;
- snapshot de frame em PPM para validação de desenvolvimento.

## Decisão técnica registrada

`docs/ADR-001-renderer-text-svg-stack.md` registra as decisões internas de renderer/texto/SVG:

- `cosmic-text = 0.16.0` foi selecionado por compatibilidade com o MSRV Rust 1.85 do Nexxus e por fornecer shaping/fallback/rasterização em Rust;
- versões posteriores inspecionadas exigiam Rust 1.89 e não foram adotadas para não elevar silenciosamente o MSRV;
- `resvg = 0.45.1` foi adotado com features padrão desabilitadas para o contrato SVG atual;
- FreeType/HarfBuzz via FFI não foram transformados em dependência direta, pois a solução selecionada satisfaz o contrato atual e reduz fronteiras `unsafe`/FFI;
- o crate usa `#![forbid(unsafe_code)]`;
- o arquivo da fonte Hack e sua licença permanecem responsabilidade da Etapa 08 — Visual Assets.

A ADR é decisão interna de implementação e não altera requisito normativo do Nexxus.

## Dependências diretas da etapa

- `cosmic-text = 0.16.0`;
- `resvg = 0.45.1` com `default-features = false`.

Dependências foram avaliadas por compatibilidade, MSRV, licença, footprint, manutenção e impacto arquitetural antes da adoção.

## Testes e validações

### Rust e qualidade

No workflow validado:

- `cargo build --workspace --release`: aprovado;
- `cargo fmt --package nexxus-ui -- --check`: aprovado;
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: aprovado;
- `cargo test --workspace --all-features`: aprovado;
- `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --all-features --no-deps`: aprovado;
- **8 testes de integração aprovados**.

Cenários cobertos:

- escala fracionária e bordas compartilhadas;
- layout flex determinístico;
- ativação/foco por pointer;
- edição UTF-8;
- semântica de acessibilidade;
- proibição de superfície estrutural translúcida;
- rasterização SVG;
- falha segura em clip stack inválida.

### Auditorias

- Shell 100% POSIX: aprovado;
- nenhum GTK/Qt no crate: aprovado;
- nenhum tipo/acoplamento concreto X11/Wayland no crate: aprovado;
- build executado como usuário não-root: aprovado.

### CI real

**Workflow da branch validada:** `31990830331`

- Arch Linux current: SUCCESS;
- Debian Trixie: SUCCESS;
- delivery: SUCCESS.

**Workflow pós-merge em `main`:** `31991101667`

- Arch Linux current: SUCCESS;
- Debian Trixie: SUCCESS;
- delivery: SKIPPED por design em `main`, pois o snapshot já havia sido produzido/versionado na branch validada.

## Footprint inicial

Medição do harness em cenário Debian validado:

- `max_rss_kib=6376`;
- `elapsed_s=0.36`;
- `binary_bytes=5107200`;
- `frame_bytes=806415`.

A medição se refere ao harness da Etapa 07, não ao footprint futuro do desktop completo.

## Build, staging e empacotamento

Entregues:

- `scripts/build-install-arch.sh` — Shell 100% POSIX;
- `scripts/build-install-debian.sh` — Shell 100% POSIX;
- helpers POSIX compartilhados;
- `scripts/check.sh`;
- auditoria POSIX e auditoria de fronteiras;
- medição automática de footprint;
- manifesto `manifests/etapa-07.conf`;
- autoprovisionamento das dependências do cenário;
- build como usuário normal;
- testes e staging isolado;
- falha segura e códigos de saída coerentes.

### Pacote nativo

**N/A nesta etapa.** `nexxus-ui` é uma biblioteca interna e `nexxus-ui-demo` é apenas harness de desenvolvimento. O manifesto usa `NEXXUS_INSTALLABLE=0`; não foi criado pacote vazio/artificial e não houve instalação direta no host. O payload será empacotado quando consumido por componente instalável pertinente.

A fonte genérica DejaVu aparece somente como dependência de build/harness dos cenários CI para garantir que o renderer de texto possa ser exercitado em imagens mínimas sem antecipar o asset Hack da Etapa 08.

## Artefato de entrega

- `entrega/Nexxus_Etapa07_Nexxus_UI_Core_0.1.0.tar.gz`;
- SHA-256: `effe50d95584d2e49b34252404cafd21f063b3b332361e3cb41d3898f405731e`;
- `Cargo.lock` validado e versionado;
- `metrics/footprint.txt` validado e versionado.

## Incidentes encontrados e corrigidos

1. **Borrow do frame no renderer:** o primeiro build detectou empréstimo incompatível durante clear; a geometria do frame passou a ser capturada antes da mutação.
2. **Fixture SVG:** raw string do teste encerrava antes do esperado por conter `#`; delimitador foi corrigido.
3. **Clippy:** o tema passou a derivar `Default`, e o borrow temporário de `cosmic-text` passou a ser encerrado por escopo léxico.
4. **Medição de footprint:** `run_logged` precisava do diretório de log de métricas criado antes de executar o helper; o wrapper foi corrigido.
5. **Imagem CI sem fonte:** containers mínimos não possuíam fonte alguma; DejaVu foi adicionado somente aos cenários de build/harness, preservando Hack/Visual Assets para a Etapa 08.

Todos os incidentes foram corrigidos e revalidados em Arch e Debian.

## Critérios de aceite

- UI própria sem GTK/Qt: **ATENDIDO**;
- widgets essenciais renderizam: **ATENDIDO**;
- widgets essenciais recebem input normalizado: **ATENDIDO**;
- escala/DPI previsível: **ATENDIDO**;
- API reutilizável e backend-neutral: **ATENDIDO**;
- superfícies estruturais dark e opacas: **ATENDIDO**;
- sem efeitos visuais proibidos: **ATENDIDO**;
- texto, imagem, SVG e clipping: **ATENDIDO**;
- foco e hit-testing: **ATENDIDO**;
- API preparada para acessibilidade: **ATENDIDO**;
- Hack como fonte semântica padrão: **ATENDIDO**;
- harness: **ATENDIDO**;
- medição inicial de footprint: **ATENDIDO**;
- Shell POSIX Arch/Debian: **ATENDIDO**;
- CI real Arch/Debian: **ATENDIDO**;
- snapshot + SHA-256: **ATENDIDO**;
- publicação da implementação em `main`: **ATENDIDO**.

## Fora do escopo preservado

Não foram implementados nesta etapa:

- Visual Assets definitivos;
- catálogo definitivo de ícones/wallpapers;
- Window Chrome;
- painel e plugins;
- menu de aplicações;
- Settings;
- File Manager;
- backend Wayland completo;
- temas GTK/Qt;
- qualquer outro módulo da Etapa 08 ou posterior.

## Pendências

Nenhuma pendência bloqueante da Etapa 07.

## Próxima etapa

- **ETAPA ATUAL:** 07 — Nexxus UI Core
- **STATUS:** VALIDADO / ENTREGUE / PUBLICADO
- **PRÓXIMA ETAPA RECOMENDADA:** Etapa 08 — Visual Assets
- **NOVA CONVERSA:** `NEXXUS FASE 08 — Visual Assets`
- **OBJETIVO:** desenvolver o catálogo versionado de assets visuais do Nexxus — ícones simbólicos SVG, fallbacks, wallpapers iniciais, manifesto/licenças e disponibilização da fonte Hack — consumindo os contratos do `nexxus-ui` sem alterar sua arquitetura.
- **DEPENDÊNCIAS DISPONÍVEIS:** Etapas 01 a 07 publicadas; `nexxus-ui` 0.1.0 validado.

A Etapa 08 deve ser iniciada exclusivamente em nova conversa.
