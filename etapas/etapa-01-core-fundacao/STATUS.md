# Estado da Etapa 01

Data-base: 2026-08-16

| Área | Estado |
|---|---|
| Workspace Cargo | implementado; compilação local bloqueada por ausência de toolchain/rede |
| `nexxus-core` | implementado; revisão estática em andamento |
| Module Registry | dependências/capabilities/ciclos implementados; testes escritos |
| Lifecycle + rollback | preflight global, descriptor matching e rollback implementados; testes escritos |
| Event Bus tipado | implementado; teste escrito |
| XDG/runtime paths | validação `0700`, ownership e rejeição de symlink implementadas |
| IPC framing/versionamento | frame completo, limite 1 MiB e negociação implementados |
| Unix Domain Socket | endpoint privado, stale-socket handling e cleanup por dev+inode implementados |
| Config TOML/atomicidade | escrita atômica, schema e limite 4 MiB implementados |
| Backend API abstrata | implementada; nenhum backend concreto |
| Logging | tracing interno mínimo + logs dos wrappers; política evolutiva |
| ABI dinâmica de plugins | deliberadamente não implementada nesta revisão |
| Build Arch/Debian | wrappers POSIX e manifesto implementados |
| Payload instalável | inexistente nesta etapa (`NEXXUS_INSTALLABLE=0`) |
| Empacotamento/instalação | N/A enquanto não houver payload; não simulado |
| Performance baseline | pendente de binário/toolchain executável |
| Módulos posteriores | não desenvolvidos |

## Validações executáveis sem Rust

- parse/sintaxe dos manifests TOML;
- `sh -n` e auditoria de bashisms dos wrappers;
- inspeção de paths/estrutura;
- verificação de ausência de `unsafe` explícito;
- revisão de segredos/temporários antes de publicação.

## Bloqueio ambiental local

O container desta conversa não possui `rustc`/`cargo` e não consegue resolver os repositórios Debian. Portanto nenhum status `COMPILADO`, `TESTADO`, `EMPACOTADO`, `INSTALADO` ou `VALIDADO` será inferido localmente. A publicação de checkpoint no GitHub pode acionar CI externo para obter evidência real de compilação/testes sem falsificar estado local.
