# NEXXUS — HANDOFF — ETAPA 12 — XDG Application Index

**Estado:** `VALIDADO`

## Identificação

- **Projeto:** Nexxus
- **Etapa:** 12 — XDG Application Index
- **Módulo:** `nexxus-xdg-application-index`
- **Versão:** 0.1.0
- **Repositório:** `https://github.com/mintonogueira/nexxus-de`
- **Branch de validação:** `etapa-12-xdg-application-index-impl`
- **Pull request:** #14
- **Commit fonte validado:** `bf11fb468a2f6de8d1666f35780e4de95c235d16`
- **Commit de entrega gerado pelo CI:** `322fd5917b0a2d920c41125fb0d12fafd21b903b`
- **GitHub Actions:** run `32007086533`

## Implementação entregue

O crate `nexxus-xdg-application-index` implementa o catálogo comum de aplicações sem UI própria.

### Descoberta e precedência

- resolve raízes XDG de dados do usuário e do sistema;
- inclui explicitamente exports Flatpak do usuário e do sistema;
- não consulta `apt`, `pacman` nem o CLI `flatpak` como fonte do índice;
- calcula Desktop File ID a partir do caminho relativo;
- respeita precedência entre raízes e mascaramento por `Hidden=true`;
- limita cada `.desktop` a 2 MiB antes do parse e não percorre symlinks de diretório.

### Metadados

- interpreta `Name`, `Exec`, `Icon`, `Categories`, `Keywords`, `NoDisplay`, `Hidden`, `OnlyShowIn`, `NotShowIn` e `DBusActivatable` necessários ao contrato da etapa;
- mantém entradas `NoDisplay` no catálogo, mas fora das views visíveis;
- normaliza categorias principais XDG e usa `Other` quando nenhuma principal é encontrada;
- preserva `Icon=` externo como caminho absoluto ou nome XDG;
- usa `nexxus-assets` da Etapa 08 somente como fallback quando o ícone oficial está ausente.

### Exec seguro

`ExecTemplate` valida a linha `Exec` e expõe `LaunchCommand { program, arguments }`. O índice não executa aplicações, não chama shell e não produz `sh -c`. Arquivos, URLs, nome, ícone e caminho do desktop file são expandidos como elementos argv segundo o tipo do field code suportado. Códigos desconhecidos invalidam a entrada; códigos depreciados são descartados.

### API comum

- `ApplicationIndexConfig`;
- `ApplicationRoot` / `ApplicationSource`;
- `scan()`;
- `IndexSnapshot`;
- `ApplicationRecord` / `DesktopId`;
- `MainCategory` / `IconReference`;
- `ExecTemplate` / `LaunchCommand` / `LaunchContext`;
- `ApplicationIndexService`;
- `ApplicationIndexEvent::Changed(IndexDelta)`;
- lookup por ID, enumeração visível, categorias, busca textual comum e diagnósticos.

A busca desta etapa é intencionalmente simples. Ranking/fuzzy search específico do Application Finder permanece fora do escopo e pertence à etapa correspondente.

### Atualização dinâmica

O serviço usa eventos de filesystem como gatilho e mantém o scan XDG como fonte autoritativa. Bursts de eventos são agrupados por debounce curto e produzem um novo snapshot determinístico apenas quando existe alteração real. Consumidores recebem `IndexDelta` com IDs adicionados, removidos e modificados.

## Dependências técnicas

- `freedesktop-desktop-entry` 0.8.1, com features padrão desabilitadas para não tornar gettext uma dependência de sistema desta etapa;
- `notify` 8.2.0 para monitoramento de filesystem;
- `thiserror` 2 para erros tipados;
- `nexxus-assets` da Etapa 08 para fallbacks de ícones;
- `tempfile` somente em testes.

As escolhas estão registradas em `docs/ADR-001-xdg-index-engine.md`. Nenhuma dependência opcional do Nexxus foi promovida a obrigatória fora deste módulo.

## Build, scripts e empacotamento

- `scripts/build-install-arch.sh`: Shell POSIX, cenário Arch Linux;
- `scripts/build-install-debian.sh`: Shell POSIX, cenário Debian;
- autoprovisionamento das dependências ausentes pelo gerenciador nativo;
- compilação e testes como usuário normal;
- staging isolado;
- `NEXXUS_INSTALLABLE=0` porque a etapa entrega biblioteca/serviço integrável, sem payload executável independente.

Consequentemente, pacote binário nativo e instalação final são `N/A` para esta etapa; não foi criada instalação artificial apenas para satisfazer o pipeline.

## Testes e validações

O GitHub Actions run `32007086533` concluiu com sucesso:

- `debian-trixie` — sucesso;
- `archlinux-current` — sucesso;
- `delivery` — sucesso.

O pipeline executou build release, auditoria POSIX, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, testes Rust e `cargo doc`.

A suíte cobre, entre outros pontos:

- entry válida e entry com `Exec` inválido;
- localização `pt_BR`;
- categorias e fallback de ícone;
- `Hidden=true` de maior precedência mascarando cópia inferior;
- `NoDisplay` presente no catálogo mas ausente das views visíveis;
- export Flatpak sem consulta ao CLI;
- criação de `.desktop` observada dinamicamente sem reiniciar o serviço;
- segurança do modelo `Exec` shell-free e expansão de listas em argv separados.

Durante a primeira execução do CI, `clippy -D warnings` apontou duas questões puramente internas (`only_used_in_recursion` e `question_mark`). Ambas foram corrigidas sem alterar o contrato funcional, e o run final acima passou integralmente.

## Snapshot de entrega

Arquivo:

`Nexxus_Etapa12_XDG_Application_Index_0.1.0.tar.gz`

SHA-256:

`f9a4f5510a57f4825a6b9dc9831296e167c58916ac5f86ae750233e155db9f99`

O snapshot e seu arquivo `.sha256` foram gerados e versionados automaticamente pelo job `delivery` após os cenários Debian e Arch passarem.

## Segurança e limites

- `#![forbid(unsafe_code)]` no crate desta etapa;
- nenhuma execução de shell para `Exec`;
- nenhuma dependência de package manager para descoberta de aplicações;
- arquivos excessivamente grandes são recusados com diagnóstico;
- erros de entries individuais não derrubam o índice;
- erros do watcher não invalidam o último snapshot válido;
- nenhuma funcionalidade visual foi implementada aqui.

## Arquivos principais

- `Cargo.toml`
- `Cargo.lock`
- `crates/nexxus-xdg-application-index/Cargo.toml`
- `crates/nexxus-xdg-application-index/src/{lib,category,config,exec,icon,model,scanner,service}.rs`
- `crates/nexxus-xdg-application-index/tests/application_index.rs`
- `manifests/etapa-12.conf`
- `scripts/build-install-arch.sh`
- `scripts/build-install-debian.sh`
- `scripts/check.sh`
- `scripts/check-posix.sh`
- `scripts/create-delivery.sh`
- `docs/ADR-001-xdg-index-engine.md`
- `README.md`
- `STATUS.md`
- `CHANGELOG.md`
- `entrega/Nexxus_Etapa12_XDG_Application_Index_0.1.0.tar.gz`
- `entrega/Nexxus_Etapa12_XDG_Application_Index_0.1.0.tar.gz.sha256`
- `.github/workflows/etapa-12-xdg-application-index.yml`

## Limites de escopo preservados

Não foram implementados nesta etapa:

- Desktop Shell;
- menu visual de aplicações;
- menu de contexto do desktop;
- Application Finder visual;
- Application Menu visual.

Esses consumidores apenas receberão o contrato produzido por esta etapa em suas próprias conversas.

## Próxima etapa recomendada

- **ETAPA ATUAL:** 12 — XDG Application Index
- **STATUS:** VALIDADO
- **PRÓXIMA ETAPA:** 13 — Desktop Shell
- **NOVA CONVERSA:** `NEXXUS FASE 13 — Desktop Shell`
- **OBJETIVO:** implementar a superfície de desktop definida no Plano Mestre consumindo o XDG Application Index validado, sem reabrir a implementação interna desta etapa.

A Etapa 13 não é iniciada neste contexto.
