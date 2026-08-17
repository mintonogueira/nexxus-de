# Nexxus Visual Assets

Pacote visual da Etapa 08.

## Contrato

- ícones internos são SVGs simbólicos 24×24 com `viewBox="0 0 24 24"`;
- `#FFFFFF` é o token cromático simbólico obrigatório e deve ser substituído pela cor semântica da paleta antes da renderização;
- ícones de aplicações externas declarados em `.desktop` são preservados e **não** devem passar pelo recolorizador simbólico;
- wallpapers são SVGs 1920×1080, totalmente locais, sem filtros, scripts ou referências externas;
- fonte padrão: `Hack`, fornecida pela distribuição, não vendorizada;
- raiz runtime do pacote: `/usr/share/nexxus/assets`.

A resolução de nomes e fallbacks é definida pelos manifests em `assets/manifest.toml` e `assets/manifests/`.
