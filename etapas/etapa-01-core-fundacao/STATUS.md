# Estado da Etapa 01

Data-base: 2026-08-16

| Área | Estado |
|---|---|
| Workspace Cargo | **COMPILADO** em Arch Linux current e Debian Trixie pela CI canônica |
| `nexxus-core` | compilado; Clippy/testes/rustdoc aprovados |
| Module Registry | implementado e testado: compatibilidade, duplicidade, dependências, capabilities e ciclos |
| Lifecycle + rollback | implementado e testado, incluindo preflight global e preservação do estado `Failed` |
| Event Bus tipado | implementado e testado |
| XDG/runtime paths | implementado e testado; rejeita symlink/permissões inseguras |
| IPC framing/versionamento | implementado e testado |
| Unix Domain Socket | endpoint privado implementado; socket ativo, stale socket e symlink testados |
| Config TOML/atomicidade | implementado e testado; limite de 4 MiB e escrita transacional |
| Backend API abstrata | implementada; nenhum backend concreto nesta etapa |
| Logging | instrumentação estrutural mínima via `tracing`; subscriber pertence ao processo futuro que hospedar o Core |
| ABI dinâmica de plugins | deliberadamente não congelada nesta etapa; contratos de isolamento permanecem abstratos |
| POSIX wrappers | auditoria estática aprovada; fluxos Arch/Debian executados na CI como usuário normal |
| Build Release | **APROVADO** nos dois cenários da CI |
| rustfmt | **APROVADO** |
| Clippy `-D warnings` | **APROVADO** |
| Testes Rust | **APROVADO** nos dois cenários |
| Rustdoc `-D warnings` | **APROVADO** |
| Staging | executado nos dois cenários |
| Pacote nativo | **N/A nesta revisão**: a Etapa 01 entrega bibliotecas/contratos e não possui payload de runtime instalável |
| Instalação | **N/A nesta revisão** pelo mesmo motivo; nenhum pacote vazio é fabricado |
| GitHub | checkpoint real publicado na `main` do repositório canônico |
| Módulos posteriores | não desenvolvidos |

## Evidência de validação

Workflow: `.github/workflows/etapa-01-core.yml`  
Run aprovado: `31974201820`  
Commit validado: `42dc0a0713e4e21c772b3dac28b3edf47a0fab1a`

Os jobs `archlinux-current` e `debian-trixie` concluíram com sucesso executando os entrypoints oficiais da própria Etapa 01.

## Estado global

A fundação está **COMPILADA e TESTADA**, mas a Etapa 01 permanece **EM DESENVOLVIMENTO** até concluir documentação/ADRs, auditoria final, snapshot `.tar.gz`, SHA-256 e handoff. Não há declaração de `EMPACOTADO` ou `INSTALADO` porque não existe payload instalável nesta etapa.
