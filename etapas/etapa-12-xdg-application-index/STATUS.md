# STATUS — Etapa 12 — XDG Application Index

- **PROJETO:** Nexxus
- **ETAPA:** 12 — XDG Application Index
- **MODULO:** `nexxus-xdg-application-index`
- **VERSAO:** 0.1.0
- **STATUS:** VALIDADO_E_PUBLICADO
- **REPOSITORIO_GITHUB:** `https://github.com/mintonogueira/nexxus-de`
- **BRANCH_CANONICA:** `main`
- **BRANCH_DE_VALIDACAO:** `etapa-12-xdg-application-index-impl`
- **PR:** `#14`
- **COMMIT_FONTE_VALIDADO:** `bf11fb468a2f6de8d1666f35780e4de95c235d16`
- **COMMIT_ENTREGA:** `322fd5917b0a2d920c41125fb0d12fafd21b903b`
- **COMMIT_MERGE_MAIN:** `d022a70a0fffc9d83a903d282c700404ce221998`
- **GITHUB_ACTIONS_RUN_VALIDACAO:** `32007086533`
- **GITHUB_ACTIONS_RUN_MAIN:** `32007387601`

## Implementado e validado

- scanner XDG/Flatpak por precedência e Desktop File ID;
- mascaramento por `Hidden=true` e apresentação respeitando `NoDisplay`/desktop corrente;
- parser/validador `Exec` shell-free, com expansão para programa + argv;
- categorias XDG e ícones oficiais/fallbacks da Etapa 08;
- snapshots imutáveis, lookup, categorias, busca simples e diagnósticos;
- watcher dinâmico com debounce, rescans determinísticos e deltas;
- testes de entries válidas/inválidas, precedência, Flatpak e atualização de filesystem;
- wrappers Shell POSIX Arch/Debian e staging isolado;
- `Cargo.lock` consolidado pelo CI;
- snapshot fonte e SHA-256 gerados pelo job de entrega.

## Evidência de validação

O workflow de validação `32007086533` concluiu com sucesso `debian-trixie`, `archlinux-current` e `delivery`. Após o merge da PR #14, o workflow `32007387601` repetiu com sucesso os cenários Debian e Arch diretamente sobre a branch `main`; o job `delivery` foi corretamente ignorado em `main` porque o snapshot já havia sido produzido e versionado na validação da branch.

Snapshot:

`Nexxus_Etapa12_XDG_Application_Index_0.1.0.tar.gz`

SHA-256:

`f9a4f5510a57f4825a6b9dc9831296e167c58916ac5f86ae750233e155db9f99`

## Empacotamento

`NEXXUS_INSTALLABLE=0`: esta etapa entrega biblioteca/serviço integrável e não possui payload executável independente. Portanto pacote nativo e instalação final são `N/A`; build, testes e staging foram executados nos dois cenários exigidos.

## Encerramento

A Etapa 12 está validada e publicada na branch `main`. Nenhuma etapa posterior foi iniciada neste contexto.
