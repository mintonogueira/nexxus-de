# NEXXUS — ETAPA 09 — Window Chrome

**Versão:** 0.1.0  
**Status:** VALIDADO E PUBLICADO NA `main`  
**Data:** 2026-08-17  
**Integração inicial na `main`:** `44dcf4114baf1156e33d725ca1eacd05b8a54f8d`

## Implementado

- chrome SSD X11 inicial com titlebar, bordas e hit targets escaláveis;
- botões `tile-fit`, `maximize/restore` e `close`;
- move por titlebar e resize por bordas/cantos;
- política CSD/SSD conservadora sem dupla decoração;
- `_NET_FRAME_EXTENTS` e classificação de tipos de janela X11;
- integração com WM Core, Backend X11, Tiling Engine, Nexxus UI Core e Visual Assets;
- estados ativo/inativo/hover/press sem animações ou efeitos proibidos;
- wrappers Arch Linux e Debian em Shell POSIX com autoprovisionamento;
- testes unitários e integração X11 real em Xvfb.

## Validação final da implementação

GitHub Actions run `31996415376`: Arch Linux, Debian e delivery concluíram com sucesso. A suíte executou release build, rustfmt, Clippy com warnings negados, testes CSD/SSD, hit-testing, tile-fit/release, X11/Xvfb, maximize/restore e rustdoc.

## Empacotamento

`NEXXUS_INSTALLABLE=0`: nesta etapa o Window Chrome é biblioteca/runtime integrável e não possui payload executável independente. Pacote nativo e instalação são `N/A`; nenhum pacote vazio é fabricado.

## Limites preservados

Wayland decorations finais, Settings completo de janelas e minimizar globalmente permanecem fora do escopo da Etapa 09.
