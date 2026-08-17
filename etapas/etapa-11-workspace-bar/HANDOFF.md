# NEXXUS — HANDOFF — ETAPA 11 — Workspace Bar

- **Projeto:** Nexxus
- **Etapa:** 11 — Workspace Bar
- **Módulo:** `nexxus-workspace-bar`
- **Versão:** 0.1.0
- **Estado técnico:** VALIDADO E PUBLICADO NA `main`
- **Branch de construção:** `etapa-11-workspace-bar`
- **Validação final:** GitHub Actions run `32003637152` — Arch Linux, Debian Trixie e delivery: SUCCESS
- **Commit validado publicado na `main`:** `d866d91b723bc6747e08a7451be3ae6ef3799b94`

## Entrega funcional

A Etapa 11 entrega a barra superior suspensa de workspaces do Nexxus. O componente apresenta todos os workspaces na ordem canônica do Workspace Manager, destaca o workspace ativo, cresce/reduz conforme criação e remoção, reage a rename/activate e permite troca por clique.

A barra inclui um botão clicável de acesso às configurações de workspaces. A Etapa 11 apenas produz a ação `OpenWorkspaceSettings`; o módulo Settings correspondente permanece fora do escopo e deverá consumi-la posteriormente.

## Arquitetura final

O crate `nexxus-workspace-bar` está separado em:

- `model.rs`: espelho visual mínimo do `WorkspaceManager` e aplicação dos eventos `Created`, `Removed`, `Renamed` e `Activated`;
- `layout.rs`: geometria lógica, crescimento/redução, centralização no monitor primário e escala HiDPI;
- `input.rs`: hit-testing, hover/press/release e ações de workspace/Settings;
- `render.rs`: composição opaca usando Nexxus UI, tema e Visual Assets;
- `x11.rs`: adapter inicial X11, RandR, superfície `override_redirect`, input e upload do frame;
- `tests/`: cobertura de modelo, eventos, input, multi-monitor, HiDPI e X11/Xvfb.

O Workspace Manager da Etapa 05 continua sendo a autoridade única sobre workspaces. A Workspace Bar não cria workspaces próprios, não associa workspace rigidamente a monitor e não drena unilateralmente a fila de eventos do manager. O runtime/coordenador pode distribuir cópias dos eventos ou sincronizar por snapshot completo.

## Contratos consumidos

- Etapa 05 — `nexxus-workspaces`: `WorkspaceManager`, `WorkspaceId`, `WorkspaceEvent` e ordem/estado canônicos;
- Etapa 07 — `nexxus-ui`: geometria lógica, `ScaleFactor`, `Theme`, `DisplayList`, `SoftwareRenderer` e texto;
- Etapa 08 — `nexxus-assets`: catálogo semântico e `preferences-workspaces`;
- Etapa 04/X11: integração gráfica inicial por `x11rb`/RandR sem alterar contratos do Backend X11.

Nenhum contrato público de etapa anterior foi reescrito.

## X11 e monitor primário

A implementação utiliza uma superfície X11 `override_redirect`, marcada semanticamente como dock, para impedir que a barra seja tratada como janela normal de aplicação. RandR é consultado somente para obter a geometria do monitor primário e posicionar a barra.

Se a topologia RandR não fornecer um monitor primário utilizável, existe fallback técnico para a primeira geometria disponível/root, evitando perder a barra por topologia incompleta. O runtime pode chamar `refresh_monitor_topology()` quando receber mudança de monitores.

A decisão está registrada em `docs/ADR-001-workspace-bar-x11-primary-monitor.md`.

## UI e interação

- visual dark/opaco;
- sem blur, sombra, fade, transparência decorativa ou animação;
- workspace ativo destacado;
- estados hover/pressed imediatos, sem timers/animação;
- botão de Settings usa asset semântico Nexxus recolorível;
- hit targets e dimensões são lógicos e escalados por `ScaleFactor`;
- tudo que é visível e pertinente na barra é operável por mouse.

## Testes e validações

A validação final é o GitHub Actions run `32003637152`, concluído com sucesso nos três jobs: Arch Linux, Debian Trixie e delivery.

A suíte validou:

- build release;
- `rustfmt` do módulo;
- Clippy com warnings negados;
- testes unitários/integrados do modelo e eventos;
- sincronização create/remove/rename/activate;
- seleção exclusiva do monitor primário em cenário multi-monitor;
- hit-testing e ações de clique;
- renderização HiDPI com Visual Asset real da Etapa 08;
- criação real da superfície X11 e RandR sob Xvfb;
- rustdoc;
- auditoria dos wrappers Shell POSIX;
- staging e geração do snapshot.

Falhas intermediárias da primeira auditoria POSIX e do check de formatação foram corrigidas antes da validação final; não permanecem como problemas conhecidos.

## Build, staging e empacotamento

Existem dois pontos de entrada independentes em Shell 100% POSIX:

- `scripts/build-install-arch.sh`;
- `scripts/build-install-debian.sh`.

Ambos validam a distribuição, autoprovisionam as dependências necessárias, compilam/testam como usuário normal e preparam staging isolado. O componente é biblioteca/runtime integrável (`NEXXUS_INSTALLABLE=0`) e não possui payload executável independente nesta etapa; pacote nativo e instalação final são `N/A`, evitando pacote vazio artificial.

## Artefatos

- **ARQUIVO_COMPACTADO:** `Nexxus_Etapa11_Workspace_Bar_0.1.0.tar.gz`
- **FORMATO:** `tar.gz`
- **VERSAO_DA_ENTREGA:** `0.1.0`
- **SHA256:** `8312e2674000130951b3c91c3d93660f23634f37532d15008371090823fb1ad6`
- **PATH:** `etapas/etapa-11-workspace-bar/entrega/Nexxus_Etapa11_Workspace_Bar_0.1.0.tar.gz`
- **STATUS_ENTREGA_COMPACTADA:** VALIDADA E PUBLICADA

O snapshot contém o estado da Etapa 11 existente no momento da geração final, incluindo código, testes, scripts, manifesto, Cargo.lock, README/STATUS e ADR; caches, `target`, `.build`, `dist` e a própria pasta `entrega` são excluídos.

## Limites preservados

- Settings de Workspaces: fora do escopo;
- painel inferior: fora do escopo;
- backend Wayland definitivo: fora do escopo;
- instanciação global pela sessão e fan-out entre consumidores: responsabilidade do runtime/coordenador, não da Workspace Bar;
- nenhuma associação rígida workspace↔monitor foi introduzida.

## Publicação e rastreabilidade

- **REPOSITORIO_GITHUB:** `https://github.com/mintonogueira/nexxus-de`
- **BRANCH_CANONICA:** `main`
- **BRANCH_DE_CONSTRUCAO:** `etapa-11-workspace-bar`
- **PASTA_DA_ETAPA:** `etapas/etapa-11-workspace-bar/`
- **COMMIT_VALIDADO_MAIN:** `d866d91b723bc6747e08a7451be3ae6ef3799b94`
- **STATUS_GITHUB:** PUBLICADO
- **ARQUIVOS_PUBLICADOS:** código Rust, testes, ADR, manifests, scripts POSIX, workflow, Cargo.lock, README/STATUS e snapshot
- **PENDENCIAS_DE_PUBLICACAO:** nenhuma para a Etapa 11

## Problemas conhecidos

Nenhum problema funcional conhecido dentro dos critérios de aceite da Etapa 11.

## Próxima etapa recomendada

**ETAPA 12 — XDG Application Index**.

Objetivo: criar o índice unificado e dinâmico de aplicações para Menu, Desktop e Application Finder, lendo entradas `.desktop` nos caminhos XDG e exports Flatpak, com API comum e atualização sem logout.

- **NOVA CONVERSA:** `NEXXUS FASE 12 — XDG Application Index`
- **PASTA_PROXIMA_ETAPA:** `etapas/etapa-12-xdg-application-index/`
- **STATUS_PROXIMA_ETAPA:** PRONTA_PARA_INICIAR

A Etapa 12 não é iniciada neste handoff.
