# Arquitetura — Etapa 01

## Fronteira

O Core oferece mecanismos e contratos; etapas posteriores oferecem funcionalidades. Nenhum handle X11/Wayland nativo atravessa a fronteira do backend.

```text
módulos futuros
     |
     v
nexxus-core ---- nexxus-protocol
     |                |
     |                +-- Unix Domain Socket / framing versionado
     |
     +---- nexxus-config
     |
     +---- nexxus-backend-api
                 |
                 +-- backend X11 futuro
                 +-- backend Wayland futuro
```

## Registry e lifecycle

O registry resolve dependências diretas e capabilities semânticas antes do lifecycle. Capacidades com múltiplos provedores exigem seleção explícita. O lifecycle executa preflight completo antes de inicializar qualquer módulo, inicia em ordem de dependência e encerra/rollback em ordem reversa. Falha primária permanece observável como `Failed` mesmo após cleanup defensivo.

## Configuração

`ConfigEnvelope<T>` carrega `schema_version` e dados tipados. Schema futuro incompatível é rejeitado. Um documento possui limite defensivo de 4 MiB. Escritas usam temporário exclusivo `0600`, `fsync`, `rename` no mesmo diretório e sincronização do diretório pai.

## Runtime paths

Paths seguem XDG quando válidos. O namespace runtime é tratado como fronteira de confiança: diretório deve pertencer ao UID corrente, negar acesso de grupo/outros e não ser symlink. O fallback temporário é privado e somente usado quando `XDG_RUNTIME_DIR` não está disponível.

## IPC

O wire interno usa prefixo big-endian de quatro bytes para tamanho do JSON, com limite de 1 MiB antes da alocação do payload. Major diferente é incompatível; minor é negociado pelo menor valor. O listener local exige diretório privado e socket `0600`; não remove symlinks, arquivos comuns ou sockets de outro usuário. Um socket stale do mesmo usuário só é substituído quando a tentativa de conexão confirma ausência de listener.

O formato é **interno e versionado**, não uma ABI pública imutável.

## Build e distribuição

A Etapa 01 define a infraestrutura comum de build: dois entrypoints POSIX independentes para Arch e Debian, manifesto por etapa, build/testes sob usuário normal, staging isolado e separação explícita de estados. Nesta revisão não existe payload de runtime; portanto não é produzido pacote vazio. A etapa que introduzir o primeiro payload instalável deverá ativar o driver nativo correspondente sem criar um pipeline paralelo.
