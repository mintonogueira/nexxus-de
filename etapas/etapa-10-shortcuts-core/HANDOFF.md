# NEXXUS — HANDOFF — ETAPA 10 — Shortcuts Core

- **Projeto:** Nexxus
- **Etapa:** 10 — Shortcuts Core
- **Módulo:** `nexxus-shortcuts`
- **Versão:** 0.1.0
- **Estado técnico:** VALIDADO E PUBLICADO NA `main`
- **Branch de construção:** `etapa-10-shortcuts-core-impl`
- **Validação final registrada:** GitHub Actions run `32000339497` — Arch Linux, Debian e delivery: SUCCESS
- **Integração inicial na `main`:** `cbea8c233557b3643ac4f9405869626afe283574`

## Entrega funcional

A etapa entrega o núcleo backend-neutral de atalhos globais configuráveis do Nexxus. O módulo contém registry de comandos/bindings, defaults normativos, parsing/canonicalização de combinações, captura de nova combinação, detecção de conflitos, rebind transacional, persistência TOML versionada, dispatch desacoplado e adaptador inicial de grabs X11.

`Super` isolado é tratado como `ModifierTap`: só dispara após pressionar/soltar sem participar de outro chord. Assim, `Super+F`, `Super+T` e demais combinações não geram um segundo acionamento ao liberar `Super`.

`F11` é rejeitado como binding global e permanece reservado às aplicações, conforme requisito normativo.

## Arquitetura final da etapa

O crate `nexxus-shortcuts` está dividido em:

- `model.rs`: `Modifier`, `Key`, `KeyChord`, `Trigger`, parser e canonicalização;
- `input.rs`: `ShortcutRecognizer`, `ShortcutCapture` e semântica de modifier tap/chords;
- `command.rs`: catálogo de `CommandId`, `CommandDescriptor` e `CommandTarget`;
- `registry.rs`: bindings, conflitos, rebind, defaults, persistência e dispatch;
- `x11.rs`: mapeamento X11, passive grabs, lock masks, rollback e liberação;
- `tests/x11_grabs.rs`: validação de grabs reais em Xvfb.

O dispatch segue o contrato `Trigger -> CommandId -> CommandDescriptor -> CommandTarget`. O host da sessão implementa `ShortcutDispatchSink` e encaminha o comando ao módulo proprietário; Shortcuts Core não incorpora estado interno de WM, workspaces, tiling, launchers, sessão, captura, áudio ou brilho.

## Contratos consumidos

- Etapa 01: `nexxus-config`, `ConfigEnvelope` e `TomlConfigStore` para persistência atômica/versionada;
- Etapa 06: `TILE_FIT_ACTION_ID` e `TilingAction::TileFit` para preservar o contrato oficial de tile-fit;
- X11: `x11rb` 0.14 para os grabs iniciais.

Nenhum contrato público de etapa anterior foi reescrito.

## Defaults entregues

- `Super` — menu de aplicações;
- `Super+F` — Application Finder;
- `Super+T` — tile-fit;
- `Ctrl+Alt+T` — Nexxus Terminal;
- `Alt+Tab` — alternância de janelas da workspace atual;
- `Super+Tab` / `Super+Shift+Tab` — navegação MRU de workspaces;
- `Super+L` — lock;
- `Ctrl+Esc` — menu do desktop;
- `Ctrl+Alt+Del` / `Ctrl+Shift+Esc` — Bashtop;
- `Alt+F4` — fechar janela focada;
- `Super+Left/Right` — workspace anterior/próxima;
- `Super+Shift+Left/Right` — mover janela focada entre workspaces;
- `Print`, `Alt+Print`, `Shift+Print` — comandos lógicos de captura;
- teclas XF86 de volume/mídia e brilho pertinentes.

A implementação concreta dos consumidores que pertencem a etapas futuras permanece fora desta etapa; os comandos lógicos e contratos de dispatch já estão disponíveis.

## X11

O adaptador consulta o keyboard map e modifier map reais do servidor, resolve keycodes, descobre os masks de Alt/Super e expande grabs para estados de Caps Lock, Num Lock e Scroll Lock. Falha de grab é explícita e aciona rollback dos grabs instalados pela tentativa corrente.

Para um modificador com binding isolado, o passive grab no próprio modificador inicia o active keyboard grab; chords que usam esse modificador não recebem passive grabs redundantes. A semântica continua pertencendo ao recognizer backend-neutral.

A decisão está registrada em `docs/ADR-001-x11-grabs.md`.

## Testes e validações

A validação final do handoff é o GitHub Actions run `32000339497`, concluído com sucesso em Arch Linux, Debian Trixie e delivery.

A suíte validou:

- build `--release`;
- `rustfmt` limitado ao módulo da etapa;
- Clippy com `-D warnings`;
- 15 testes unitários, todos aprovados;
- teste real `installs_and_releases_default_grabs` sob Xvfb, aprovado;
- rustdoc com warnings negados;
- reconhecimento de Super isolado e supressão após chords;
- parsing/canonicalização;
- F11 não global, inclusive após reconfiguração;
- conflitos sem sobrescrita silenciosa;
- persistência e round-trip versionado;
- dispatch de comandos;
- resolução X11 dos defaults e lock masks;
- instalação e remoção de passive grabs X11.

## Build, staging e empacotamento

Existem dois pontos de entrada independentes em Shell 100% POSIX:

- `scripts/build-install-arch.sh`;
- `scripts/build-install-debian.sh`.

Ambos autoprovisionam dependências, compilam/testam como usuário normal e preparam staging isolado. O módulo é uma biblioteca/runtime integrável (`NEXXUS_INSTALLABLE=0`), sem payload executável independente; por isso pacote nativo e instalação são `N/A` nesta etapa, em vez de criar pacote vazio artificial.

## Artefatos

- `entrega/Nexxus_Etapa10_Shortcuts_Core_0.1.0.tar.gz`;
- `entrega/Nexxus_Etapa10_Shortcuts_Core_0.1.0.tar.gz.sha256`.

O SHA-256 final deve ser lido do arquivo `.sha256` adjacente ao snapshot para evitar referência circular entre o handoff e o próprio arquivo compactado que contém este documento.

## Limites preservados

- UI completa de configuração de atalhos: fora do escopo desta etapa;
- backend Wayland final / XDG GlobalShortcuts: fora do escopo desta etapa;
- implementação de menu, Application Finder, Terminal, Session Lock, Screenshot, áudio e brilho: pertence às respectivas etapas futuras;
- esta etapa fornece somente contratos/dispatch para esses consumidores futuros.

## Publicação e rastreabilidade

- **REPOSITORIO_GITHUB:** `https://github.com/mintonogueira/nexxus-de`
- **BRANCH:** `etapa-10-shortcuts-core-impl`
- **PASTA_DA_ETAPA:** `etapas/etapa-10-shortcuts-core/`
- **COMMIT_VALIDADO_BRANCH:** `cbea8c233557b3643ac4f9405869626afe283574`
- **COMMIT_MAIN:** `cbea8c233557b3643ac4f9405869626afe283574` — integração inicial do estado validado
- **STATUS_GITHUB:** PUBLICADO
- **ARQUIVOS_PUBLICADOS:** código Rust, testes, ADR, manifests, scripts POSIX, workflow, Cargo.lock, handoff e snapshot
- **ARTEFATOS_PARA_DOWNLOAD:** snapshot `.tar.gz` e arquivo `.sha256`
- **ARQUIVO_COMPACTADO:** `Nexxus_Etapa10_Shortcuts_Core_0.1.0.tar.gz`
- **SHA256:** consultar `Nexxus_Etapa10_Shortcuts_Core_0.1.0.tar.gz.sha256`
- **PENDENCIAS_DE_PUBLICACAO:** nenhuma para a Etapa 10

## Próxima etapa

**ETAPA 11 — Workspace Bar**. A implementação deverá começar exclusivamente em nova conversa após este encerramento.

- **PASTA_PROXIMA_ETAPA:** `etapas/etapa-11-workspace-bar/` — deve existir apenas como preparação estrutural;
- **STATUS_PROXIMA_ETAPA:** PRONTA_PARA_INICIAR, sem implementação nesta conversa.
