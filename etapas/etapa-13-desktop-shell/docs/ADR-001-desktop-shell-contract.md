# ADR-001 — Contrato interno do Desktop Shell

**Status:** ACEITO INTERNAMENTE  
**Data:** 2026-08-17

## Decisões

1. O Desktop Shell mantém somente estado pertencente ao desktop: wallpaper e launchers/posições. A persistência usa `nexxus-config` em vez de criar outro formato de escrita.
2. O índice de aplicações permanece autoridade exclusiva da Etapa 12. Launchers persistem Desktop File IDs em texto e são revalidados contra o snapshot corrente antes de aparecer ou executar.
3. `Ctrl+Esc` é consumido como `CommandTarget::Shell(ShellAction::DesktopMenu)` da Etapa 10; a Etapa 13 não cria grabs concorrentes.
4. A ação Create Folder cria somente um diretório imediato dentro do XDG Desktop. Listagem/gestão ampla de arquivos continua fora do escopo e pertence ao File Manager.
5. Terminal, File Manager e Desktop Settings são pedidos semânticos devolvidos ao coordenador da sessão, porque os respectivos módulos possuem etapas próprias.
6. Aplicações com `Exec` são iniciadas por argv validado, nunca por shell. Entradas `DBusActivatable` usam `org.freedesktop.Application` na session bus.
7. O adapter X11 usa uma superfície única `override_redirect`, marcada `_NET_WM_WINDOW_TYPE_DESKTOP` e mantida abaixo das janelas normais. RandR só delimita regiões de monitor; o menu não é replicado.
8. O módulo é um runtime integrável (`NEXXUS_INSTALLABLE=0`). O Session Runtime será responsável por instanciá-lo no desktop completo; não será criado pacote vazio ou executável artificial nesta etapa.
9. Wallpapers raster são decodificados com `image` apenas nos formatos PNG/JPEG/WebP; SVG usa diretamente o renderer da Etapa 07. A dependência é restrita a decodificação e não introduz toolkit de UI.
10. `zbus` é usado somente para ativação D-Bus padrão de aplicações; não altera contratos públicos do Nexxus.

Nenhuma destas decisões altera requisito funcional ou visual vigente; são escolhas internas de implementação dentro da autonomia do Aditivo 04.
