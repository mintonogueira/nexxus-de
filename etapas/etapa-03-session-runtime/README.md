# Nexxus — Etapa 03 — Session Runtime

Implementação da Etapa 03 do Projeto Nexxus.

## Escopo implementado

- crate/binário `nexxus-session`;
- seleção explícita `x11`/`wayland`, sem fallback silencioso;
- configuração versionada sobre `nexxus-config`;
- resolução/validação de paths XDG e runtime privado sobre `nexxus-core`;
- preparação segura de `XDG_CURRENT_DESKTOP`, `XDG_SESSION_DESKTOP` e `XDG_SESSION_TYPE` para processos de integração;
- endpoint IPC privado `session.sock` sobre `nexxus-protocol`;
- orquestração determinística do lifecycle por `ModuleRegistry`/`LifecycleManager`;
- adapter de lifecycle para `nexxus-wm`, sem duplicação da lógica do WM;
- shutdown em ordem inversa de dependências;
- status mínimo da sessão para diagnóstico;
- testes de seleção, bootstrap, rollback e shutdown.

## Limite intencional desta etapa

Nenhum backend gráfico concreto pertence à Etapa 03. Por isso o binário instalado consegue validar configuração e ambiente com `--check`, mas recusa iniciar uma sessão gráfica real até que um backend concreto seja fornecido por etapa posterior. Essa recusa é explícita e não troca X11 por Wayland (ou vice-versa) silenciosamente.

## Uso atual

```text
nexxus-session --backend=x11 --check
nexxus-session --backend=wayland --check
```

Configuração padrão: `$XDG_CONFIG_HOME/nexxus/session.toml` ou `~/.config/nexxus/session.toml`.
