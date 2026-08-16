# Changelog

## 0.1.0-dev — 2026-08-16

### Fundação
- Workspace Rust da Etapa 01 com quatro crates backend-agnostic.
- Registry com dependências explícitas, capabilities, seleção de provider e detecção de ciclos.
- Lifecycle com verificação prévia de todas as implementações, descriptor matching e rollback reverso.
- Paths XDG/runtime endurecidos contra symlink e permissões/ownership inseguros.
- IPC local versionado com framing limitado, validação de frame completo e endpoint Unix privado.
- Configuração TOML versionada, gravação atômica e limite defensivo de 4 MiB.
- API abstrata de graphics backend sem implementação X11/Wayland.

### Engenharia/entrega
- Metadata Cargo alinhada ao `LICENSE` GPL-3.0 do repositório canônico.
- Manifesto operacional da Etapa 01.
- Wrappers POSIX separados para Arch Linux e Debian com autoprovisionamento.
- Contrato comum de build/test/staging/packaging documentado.
- ADRs E01-017 a E01-020.
- Auditoria POSIX estática adicional.

### Estado ainda pendente
- Compilação e execução da suíte Rust em ambiente com toolchain.
- Baseline de desempenho após existir artefato compilável.
- Pacote/instalação não se aplicam enquanto a etapa não possuir payload de runtime instalável.
