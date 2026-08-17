# Nexxus — Etapa 04 — Backend X11

**Status:** `VALIDADO / PUBLICADO / ENTREGUE`

A Etapa 04 entrega o primeiro backend gráfico concreto do Nexxus, conectando o `nexxus-wm` ao X11 com interoperabilidade EWMH/ICCCM e integração ao Session Runtime.

## Evidência final

- repositório canônico: `https://github.com/mintonogueira/nexxus-de`;
- branch canônica: `main`;
- PR de publicação: `#3`;
- commit validado na `main`: `081b152e863f5b91dafd481e864f7fa8658f530a`;
- workflow final na `main`: `31985046800` — SUCCESS;
- workflow técnico da branch: `31984901974` — SUCCESS;
- pacote Arch: `nexxus-backend-x11-0.1.0-1-x86_64.pkg.tar.zst`;
- pacote Debian: `nexxus-backend-x11_0.1.0_amd64.deb`;
- snapshot: `Nexxus_Etapa04_Backend_X11_0.1.0.tar.gz`;
- SHA-256 do snapshot: `f3548511e63348d4ade7590dc07d56f70a978d19d9c595be43376dee27c1b102`.

## Decisões técnicas vigentes

- binding X11: `x11rb 0.14.0`, conexão Rust pura, sem feature `allow-unsafe-code`;
- crate da etapa mantém `#![forbid(unsafe_code)]`;
- EWMH/ICCCM pertinentes são tratados no adapter X11;
- as janelas não são reparentadas nem decoradas nesta etapa, preservando CSD/SSD e a fronteira da futura Etapa 09 — Window Chrome;
- compositor X11 não é ativado porque não é tecnicamente necessário ao contrato desta etapa; nenhum efeito visual proibido foi introduzido.

## Continuidade

Próxima etapa: **Etapa 05 — Workspace Manager**. O desenvolvimento deve ocorrer em nova conversa.

Handoff: `docs/HANDOFF_FINAL_ETAPA_04.md`.
