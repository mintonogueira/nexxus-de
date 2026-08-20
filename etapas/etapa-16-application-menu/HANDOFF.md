# NEXXUS — HANDOFF — ETAPA 16 — APPLICATION MENU

**Estado:** `VALIDADO_NO_ESCOPO / BLOQUEADO_PARA_INTEGRAÇÃO`

## Identificação

- **Projeto:** Nexxus
- **Etapa:** 16 — Application Menu
- **Módulo:** `nexxus-app-menu`
- **Versão:** 0.1.0
- **Repositório:** `https://github.com/mintonogueira/nexxus-de`
- **Branch:** `etapa-16-application-menu-impl-v2`
- **Pull request:** #19
- **Base temporária da PR:** `etapa-15-panel-core-impl`
- **Commit de código validado:** `ba053f32e496213a6c5218466c4f1b0d3ca75f0e`
- **GitHub Actions validado:** run `32329631563`

## Implementação entregue

O crate `nexxus-app-menu` implementa o estado e os contratos funcionais do menu de aplicações sem reimplementar indexação XDG nem o host de plugins do painel.

### Catálogo e navegação

- consome `IndexSnapshot` da Etapa 12 como fonte autoritativa;
- busca instantânea usa `IndexSnapshot::search()`;
- seções: Favoritos, Recentes, Todos e categoria XDG;
- favoritos usam coleção determinística sem duplicatas;
- recentes usam ordem MRU com limite configurável;
- registros `NoDisplay`/não visíveis continuam filtrados pelo contrato da Etapa 12.

### Exibição

- modos `List` e `Grid`;
- tamanhos de ícone `Small`, `Medium` e `Large`;
- `MenuEntry` preserva `IconReference` oficial ou fallback entregue pela Etapa 12;
- o estado de apresentação permanece backend-neutral e não introduz GTK/Qt.

### Execução

- `launch_command()` consome o `ExecTemplate` validado pela Etapa 12;
- produz `LaunchCommand { program, arguments }` sem `sh -c` e sem shell;
- o aplicativo só entra em Recentes depois da geração válida do comando;
- execução efetiva do processo permanece responsabilidade do runtime/launcher consumidor.

### Integração com Panel Core

`ApplicationMenuPanelPlugin` implementa diretamente `nexxus_panel::PanelPlugin` e declara:

- `plugin_id`: `nexxus.application-menu`;
- `display_name`: `Application Menu`;
- API: `PluginApiVersion::CURRENT` (1.0 no estado atual da Etapa 15);
- lifecycle `load/unload` com validação da instância carregada.

Nenhum código do Panel Core foi duplicado nesta etapa.

## Testes e validação

O run `32329631563` concluiu com sucesso nos dois cenários:

- Arch Linux — sucesso;
- Debian Trixie — sucesso.

Foram validados:

- `cargo fmt --package nexxus-app-menu -- --check`;
- `cargo test --package nexxus-app-menu --all-targets`;
- `cargo clippy --package nexxus-app-menu --all-targets -- -D warnings`;
- `cargo doc --package nexxus-app-menu --no-deps` com `RUSTDOCFLAGS=-D warnings`.

A suíte cobre busca, favoritos, recentes, expansão shell-free do `Exec`, lifecycle do plugin e semântica idempotente de abertura do menu.

## Build e empacotamento

- `scripts/build-install-arch.sh` — Shell POSIX, cenário Arch Linux;
- `scripts/build-install-debian.sh` — Shell POSIX, cenário Debian;
- manifesto `manifests/etapa-16.conf` separa dependências por distribuição;
- build/teste permanecem como usuário normal nos wrappers;
- `NEXXUS_INSTALLABLE=0`: esta etapa entrega plugin/runtime integrável, sem payload executável independente; pacote binário final é `N/A` neste estágio, evitando instalação artificial.

## Decisões técnicas relevantes

1. A Etapa 16 consome diretamente os contratos públicos das Etapas 12 e 15; não cria parser XDG nem host de plugins paralelo.
2. O CI da Etapa 16 foi limitado explicitamente ao crate `nexxus-app-menu`, evitando que `cargo fmt` ou Clippy modifiquem/validem como se pertencessem a esta etapa os fontes da Etapa 15.
3. A PR é empilhada sobre a branch da Etapa 15 porque o Panel Core ainda não está integrado à `main`.

## Bloqueio de integração

A PR #18 da Etapa 15 permanece aberta. O último CI conhecido da Etapa 15 falhou em `cargo fmt --check`. Corrigir essa falha pertence à conversa/etapa do Panel Core e não foi feito aqui, conforme o isolamento estrito do Nexxus.

Consequentemente, a Etapa 16 está validada em seu próprio escopo, mas **não deve ser integrada à `main` antes de a Etapa 15 ser validada e integrada**. Depois disso, a PR #19 deve ser retargetada para `main` e receber validação final pós-rebase/retarget.

## Arquivos principais

- `Cargo.toml`
- `crates/nexxus-app-menu/Cargo.toml`
- `crates/nexxus-app-menu/src/lib.rs`
- `crates/nexxus-app-menu/tests/application_menu.rs`
- `manifests/etapa-16.conf`
- `scripts/check.sh`
- `scripts/build-install-arch.sh`
- `scripts/build-install-debian.sh`
- `.github/workflows/etapa-16-application-menu.yml`
- `README.md`
- `STATUS.md`
- `HANDOFF.md`

## Próxima ação

- **ETAPA ATUAL:** 16 — Application Menu
- **STATUS:** VALIDADO_NO_ESCOPO / BLOQUEADO_PARA_INTEGRAÇÃO
- **AÇÃO NECESSÁRIA ANTES DO MERGE:** retornar à conversa `NEXXUS FASE 15 — Panel Core`, corrigir/validar a PR #18 e integrá-la à `main`.
- **DEPOIS:** retargetar a PR #19 para `main`, executar CI final e concluir a integração da Etapa 16.

Nenhuma etapa seguinte deve ser iniciada nesta conversa antes desse bloqueio de dependência ser resolvido.
