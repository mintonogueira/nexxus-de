# Arquitetura — Etapa 15 — Panel Core

## Responsabilidade

`nexxus-panel` concentra política do painel inferior, geometria, métricas proporcionais, placement, lifecycle de plugins e persistência. A apresentação concreta continua nos backends gráficos.

## Plugin API

A API lógica inicia em `1.0`. Plugins pequenos são hospedados no processo do painel para evitar um processo pesado por item. O registry valida versão e IDs, impede duplicação e preserva a instância quando `unload` falha, evitando perder referência para recurso ainda ativo.

## Layout

Existem zonas `Start`, `Center` e `End`, mas elas são somente organização lógica. Instâncias podem ser movidas entre zonas e reordenadas livremente. A altura aceita 24–96 unidades lógicas e gera icon size, padding, spacing e hit target proporcionalmente.

## Persistência

Schema inicial `1`. O arquivo guarda altura e placement das instâncias. A gravação usa arquivo temporário, `sync_all()` e `rename()`, de modo que uma escrita parcial não substitua silenciosamente a última configuração válida.

## X11

O contrato X11 segue EWMH: o presenter deve marcar a janela como `_NET_WM_WINDOW_TYPE_DOCK` antes do map e publicar `_NET_WM_STRUT_PARTIAL` para reservar a faixa inferior. `_NET_WM_STRUT` pode ser publicado como compatibilidade adicional. O transporte X11 não pertence ao core.

## Fronteiras

Application Menu, Dock/Task Buttons e plugins funcionais permanecem fora desta etapa. GTK/Qt não são usados na UI própria do Nexxus.
