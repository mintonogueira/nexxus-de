# HANDOFF FINAL — NEXXUS ETAPA 09 — Window Chrome

## Identificação

- **Projeto:** Nexxus
- **Etapa:** 09 — Window Chrome
- **Módulo:** `nexxus-window-chrome`
- **Versão:** 0.1.0
- **Status:** VALIDADO E PUBLICADO
- **Branch de construção:** `etapa-09-window-chrome-impl`
- **Validação final da implementação:** GitHub Actions run `31996415376` — Arch, Debian e delivery: SUCCESS
- **Integração inicial na `main`:** `44dcf4114baf1156e33d725ca1eacd05b8a54f8d`

## Entrega funcional

A etapa implementa a decoração própria Nexxus para janelas X11 quando SSD é aplicável, preservando CSD externa. O chrome possui titlebar, bordas, tile-fit, maximize/restore, close, move por titlebar, resize por bordas/cantos, estados visuais imediatos e hit targets escaláveis.

## Arquitetura final da etapa

O crate está dividido em `policy.rs`, `geometry.rs`, `render.rs`, `integration.rs` e `x11.rs`. A decisão técnica registrada em `docs/ADR-001-x11-ssd-sem-reparenting.md` mantém as janelas cliente sob responsabilidade do Backend X11 da Etapa 04; a Etapa 09 cria somente superfícies próprias de decoração e delega operações ao controller existente.

## Contratos consumidos

- Etapa 02: `WindowManager`, estados, geometria, `WmCommand`, `BackendCommandSink`;
- Etapa 04: `X11Controller` para focus/move/resize/maximize/restore/fullscreen/close;
- Etapa 06: `tile_fit_active`, `release_for_manual_operation` e dispatch de planos;
- Etapa 07: `Theme`, `ScaleFactor`, `DisplayList`, renderer e primitivas geométricas;
- Etapa 08: catálogo/recoloração dos ícones `window-tile`, `window-maximize`, `window-restore` e `window-close`.

Nenhum contrato público das etapas anteriores foi reescrito.

## Testes e validações

A validação final cobre política CSD por `_GTK_FRAME_EXTENTS` e Motif hints; tipos especiais X11; prioridade de hit targets; resize com tamanho mínimo; escala de frame extents; renderização dos assets oficiais; tile-fit seguido de liberação manual para floating; integração X11 real sob Xvfb com janelas SSD/CSD; `_NET_FRAME_EXTENTS`; maximize/restore preservando geometria; `cargo build --release`; rustfmt; Clippy `-D warnings`; testes; rustdoc; ausência de GTK/Qt próprio e ausência de minimizar globalmente.

## Build e distribuição

Os pontos de entrada `scripts/build-install-arch.sh` e `scripts/build-install-debian.sh` são Shell 100% POSIX, autoprovisionam dependências, compilam como usuário normal, testam, fazem staging e falham de forma explícita. Como não existe payload executável independente, pacote nativo/instalação são `N/A` nesta etapa.

## Snapshot

- `entrega/Nexxus_Etapa09_Window_Chrome_0.1.0.tar.gz`
- `entrega/Nexxus_Etapa09_Window_Chrome_0.1.0.tar.gz.sha256`
- O pipeline de delivery regenera ambos depois deste handoff para garantir correspondência entre documentação e snapshot.

## Limites preservados

Decoração Wayland final, Settings completo de janelas e minimizar globalmente permanecem fora do escopo normativo. A ligação de `ChromeHooks` à instância canônica de sessão/tiling pertence ao host que compõe os módulos e não transfere responsabilidades do Session Runtime para esta etapa.

## Próxima etapa

**ETAPA 10 — Shortcuts Core**. Deve ser iniciada exclusivamente em nova conversa; este handoff não inicia a etapa seguinte.
