# Contratos — Etapa 02 — Window Manager Core

## Entrada: `WmEvent`

Eventos são normalizados por futuros adaptadores gráficos antes de entrarem no `nexxus-wm`.

- `WindowCreated`: registra ID, geometria, constraints e metadados.
- `WindowDestroyed`: remove a janela; repetição após remoção é tratada como evento stale.
- `WindowMapped` / `WindowUnmapped`: atualizam disponibilidade lógica para foco/visibilidade.
- `WindowGeometryChanged`: confirma geometria observada pelo backend.
- `FocusChanged`: sincroniza foco observado; alvo inexistente é stale.
- `WindowMetadataChanged`: atualiza título/identidade normalizados.

Eventos atrasados de uma janela já destruída não recriam estado e retornam `EventOutcome::IgnoredStale`.

## Saída: `WmCommand`

O core emite intenção, não chamadas de protocolo:

- `RequestFocus`;
- `RequestMove`;
- `RequestResize`;
- `RequestMaximize`;
- `RequestRestore`;
- `RequestFullscreen`;
- `RequestClose`.

`BackendCommandSink` é a fronteira mínima que um futuro adaptador/backend deverá consumir. Handles X11/Wayland não fazem parte deste contrato.

## Ownership

- `WindowManager` é a autoridade do estado lógico das janelas.
- O backend é autoridade sobre a execução física e devolve o resultado observado por eventos.
- Comandos de move/resize não alteram antecipadamente a geometria confirmada; o estado muda quando chega `WindowGeometryChanged`.
- Estados de apresentação são atualizados logicamente no pedido para que transições subsequentes sejam validadas de forma determinística.

## Erros

- criação com ID duplicado: erro;
- operação explícita sobre ID desconhecido: erro;
- atualização/evento tardio após destroy: ignorado como stale;
- sequência de apresentação inválida: erro sem consumir o snapshot de restore;
- falha do backend em `BackendCommandSink`: propagada como `WmError::Backend`.

## Versionamento

Este contrato nasce na versão `0.1.0` da Etapa 02. Mudança incompatível consumida por etapa posterior deverá ser coordenada e documentada; extensão interna compatível pode evoluir dentro da governança do Nexxus.
