# HANDOFF FINAL — NEXXUS ETAPA 09 — Window Chrome

## Identificação

- **Projeto:** Nexxus
- **Etapa:** 09 — Window Chrome
- **Módulo:** `nexxus-window-chrome`
- **Versão:** 0.1.0
- **Estado técnico:** VALIDADO EM BRANCH
- **Branch:** `etapa-09-window-chrome-impl`
- **Validação principal:** GitHub Actions run `31995844354` — Arch, Debian e delivery: SUCCESS

## Entrega funcional

A etapa implementa a decoração própria Nexxus para janelas X11 quando SSD é aplicável, preservando CSD externa. O chrome possui titlebar, bordas, tile-fit, maximize/restore, close, move por titlebar, resize por bordas/cantos, estados visuais imediatos e hit targets escaláveis.

## Arquitetura final da etapa

O crate está dividido em:

- `policy.rs`: normalização dos hints X11 e decisão CSD/SSD/sem decoração;
- `geometry.rs`: métricas, extents, hit-testing e cálculo de resize;
- `render.rs`: pintura opaca por `nexxus-ui` e consumo de SVGs da Etapa 08;
- `integration.rs`: contratos com Etapas 02, 04 e 06;
- `x11.rs`: superfícies SSD X11, `_NET_FRAME_EXTENTS`, input e sincronização.

Decisão técnica registrada em `docs/ADR-001-x11-ssd-sem-reparenting.md`: o Window Chrome não reparenta janelas cliente. O Backend X11 da Etapa 04 continua proprietário das operações físicas das janelas; a Etapa 09 cria apenas superfícies próprias de decoração e delega comandos ao controller existente.

## Contratos consumidos

- Etapa 02: `WindowManager`, estados, geometria, `WmCommand`, `BackendCommandSink`;
- Etapa 04: `X11Controller` para focus/move/resize/maximize/restore/fullscreen/close;
- Etapa 06: `tile_fit_active`, `release_for_manual_operation` e dispatch de planos;
- Etapa 07: `Theme`, `ScaleFactor`, `DisplayList`, renderer e primitivas geométricas;
- Etapa 08: catálogo e recoloração dos ícones `window-tile`, `window-maximize`, `window-restore` e `window-close`.

Nenhum contrato público das etapas anteriores foi reescrito.

## Testes e validações

A validação cobre:

- política CSD por `_GTK_FRAME_EXTENTS` e Motif hints;
- tipos especiais X11 sem decoração normal;
- prioridade dos botões sobre a área de arraste;
- resize por bordas com tamanho mínimo;
- escala de frame extents;
- renderização dos assets oficiais da Etapa 08;
- tile-fit real seguido de liberação manual para floating, provando que o tiling não prende a janela;
- integração X11 real sob Xvfb com uma janela SSD e uma CSD;
- publicação de `_NET_FRAME_EXTENTS` apenas na SSD;
- maximize/restore com preservação da geometria anterior;
- `cargo build --release`, rustfmt, Clippy `-D warnings`, testes e rustdoc;
- auditoria de fronteira: sem GTK/Qt próprio e sem minimizar globalmente.

## Build e distribuição

Há dois pontos de entrada POSIX independentes:

- `scripts/build-install-arch.sh`
- `scripts/build-install-debian.sh`

Ambos autoprovisionam dependências, compilam como usuário normal, testam, fazem staging e interrompem em falha. Como não existe payload executável independente, pacote nativo/instalação são explicitamente `N/A` nesta etapa.

## Snapshot

- `entrega/Nexxus_Etapa09_Window_Chrome_0.1.0.tar.gz`
- SHA-256 é gerado em arquivo adjacente pelo pipeline de delivery.
- O snapshot é regenerado quando o handoff final é versionado para corresponder ao estado final da etapa.

## Pendências e limites

Não são pendências da Etapa 09: decoração Wayland final, Settings completo de janelas e minimizar globalmente; esses itens permanecem fora do escopo normativo.

A integração do `ChromeHooks` com a instância canônica de sessão/tiling deverá ser feita pelo host que compõe os módulos; a Etapa 09 fornece o contrato e não incorpora responsabilidades do Session Runtime.

## Publicação

O estado técnico foi validado na branch de implementação. A referência de `main` será registrada após a integração final, sem reescrita de histórico.

## Próxima etapa

**ETAPA 10 — Shortcuts Core**. Deve ser iniciada exclusivamente em nova conversa após o encerramento/publicação desta etapa.
