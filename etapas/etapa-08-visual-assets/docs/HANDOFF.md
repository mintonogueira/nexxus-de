# HANDOFF — Nexxus Etapa 08 — Visual Assets

## Registro da etapa

- **Projeto:** Nexxus
- **Etapa:** 08 — Visual Assets
- **Módulo:** `nexxus-assets` + payload `nexxus-visual-assets`
- **Versão:** 0.1.0
- **Status:** VALIDADO
- **Repositório GitHub:** https://github.com/mintonogueira/nexxus-de
- **Branch de implementação:** `etapa-08-visual-assets`
- **PR de implementação:** #10
- **Merge em `main`:** `b407c4c0c2724625c234c37b790d40f33c5f1be1`
- **CI de validação e entrega:** workflow run `31992926930`

## Resultado entregue

A Etapa 08 formaliza os Visual Assets próprios do Nexxus sem implementar componentes das etapas posteriores. Foram entregues:

- 82 ícones simbólicos SVG 24×24;
- 10 wallpapers vetoriais dark 1920×1080;
- manifesto versionado de assets;
- fallbacks semânticos para categorias de aplicações e MIME;
- crate Rust `nexxus-assets` para catálogo, fallback e recoloração;
- registro de licenças e proveniência;
- fonte Hack consumida por dependência nativa da distribuição, sem vendorizar arquivos da fonte;
- scripts Shell 100% POSIX separados para Arch Linux e Debian;
- empacotamento binário nativo e instalação de teste do pacote gerado;
- CI Arch Linux e Debian;
- snapshot da etapa com SHA-256.

## Contrato técnico

### SVGs simbólicos

Os SVGs próprios usam `#FFFFFF` como token cromático canônico. `recolor_symbolic_svg()` substitui somente esse token por `#RRGGBB` antes de entregar os bytes ao renderer SVG já existente na Etapa 07.

Essa solução evita alterar a API da Etapa 07 dentro desta conversa e mantém o isolamento entre etapas.

### Ícones de aplicações externas

`resolve_application_icon()` preserva qualquer ícone externo válido declarado pela aplicação. O fallback Nexxus só é usado quando o ícone oficial não está disponível. Ícones externos não passam pelo recolorizador simbólico.

### Fontes

A família padrão permanece Hack. A fonte não é copiada para o repositório nem para o payload do Nexxus:

- Arch Linux: `ttf-hack`;
- Debian: `fonts-hack`.

### Runtime

Raiz instalada dos assets:

`/usr/share/nexxus/assets`

## Estrutura principal

- `assets/icons/` — SVGs simbólicos próprios;
- `assets/wallpapers/` — wallpapers distribuíveis;
- `assets/manifest.toml` — manifesto principal;
- `assets/manifests/` — regras de fallback;
- `assets/LICENSES.md` — licenças/proveniência;
- `crates/nexxus-assets/` — API Rust do catálogo;
- `packaging/arch/` — metadados Arch;
- `packaging/debian/` — metadados Debian;
- `scripts/` — validação, build, empacotamento e entrega POSIX;
- `dist/` — pacotes binários validados;
- `entrega/` — snapshot e hash;
- `.github/workflows/etapa-08-visual-assets.yml` — CI da etapa.

## Testes e validações executados

O workflow `31992926930` concluiu com sucesso nos três jobs pertinentes:

- `archlinux-current`: SUCCESS;
- `debian-trixie`: SUCCESS;
- `delivery`: SUCCESS.

A validação executou:

- auditoria POSIX dos wrappers `/bin/sh`;
- validação XML dos SVGs com `xmllint`;
- contagem exata de 82 ícones e 10 wallpapers;
- validação de `viewBox` e token cromático;
- bloqueio de scripts, imagens externas, filtros e transparência decorativa nos SVGs;
- rasterização dos ícones em 16 px e 64 px;
- rasterização reduzida dos wallpapers;
- `cargo fmt --check`;
- `cargo clippy --workspace --all-targets -- -D warnings`;
- `cargo test --workspace`;
- `cargo doc --workspace --no-deps` com warnings tratados como erro;
- staging isolado;
- geração e inspeção de pacote nativo;
- verificação de ausência de fontes vendorizadas;
- instalação do mesmo pacote gerado pelo gerenciador nativo;
- smoke test do payload instalado e resolução da fonte Hack por Fontconfig.

## Pacotes validados

### Arch Linux

- `nexxus-visual-assets-0.1.0-1-any.pkg.tar.zst`
- SHA-256: `a76b6bb00ee7db1c013df1f760d1f1f4d4ad0a86720d44a349b9f766cbfb2645`

### Debian

- `nexxus-visual-assets_0.1.0_all.deb`
- SHA-256: `6dd8d64e33150123b2fa520df87eb8daeaf498ffddb1800cd6ac62a730c66043`

## Snapshot

- `Nexxus_Etapa08_Visual_Assets_0.1.0.tar.gz`
- SHA-256: `a8ed7e5a7348b9875a09ab377d2033c9d63b7c77423d5729ff01f8ff8cb8f876`

## Decisões técnicas relevantes

A decisão interna está registrada em `docs/ADR-001-visual-assets-contract.md`:

1. recoloração por token explícito em SVG próprio;
2. preservação absoluta de ícones externos;
3. fallback apenas quando necessário;
4. Hack fornecida pela distribuição;
5. ausência de efeitos/filtros/conteúdo ativo nos SVGs distribuídos.

Nenhuma decisão normativa do Nexxus foi substituída; portanto, não foi necessário criar novo Documento Aditivo.

## Critérios de aceite

- SVGs escaláveis: **VALIDADO**;
- recoloração de assets próprios: **VALIDADO**;
- preservação de ícones oficiais de aplicações: **VALIDADO**;
- ausência de GTK/Qt ou toolkit externo no módulo: **VALIDADO**;
- licenças registradas: **VALIDADO**;
- cerca de 10 wallpapers dark: **VALIDADO — 10 entregues**;
- build/packaging Arch Linux: **VALIDADO**;
- build/packaging Debian: **VALIDADO**;
- snapshot + SHA-256: **VALIDADO**;
- publicação no repositório canônico `main`: **VALIDADO**.

## Limitações e itens deliberadamente fora do escopo

Permanecem fora desta etapa:

- branding/logo final;
- Window Chrome;
- temas GTK/Qt;
- efeitos ou animações;
- implementação de Panel, Settings, File Manager ou demais módulos consumidores.

Esses itens não constituem pendência da Etapa 08.

## Próxima etapa

- **Próxima etapa recomendada:** 09 — Window Chrome
- **Nova conversa:** `NEXXUS FASE 09 — Window Chrome`
- **Dependências disponíveis:** Nexxus UI Core da Etapa 07 e Visual Assets validados da Etapa 08.

A Etapa 09 deve consumir os contratos existentes sem alterar silenciosamente a Etapa 08.
