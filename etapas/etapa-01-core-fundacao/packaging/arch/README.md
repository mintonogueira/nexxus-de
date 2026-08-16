# Contrato de empacotamento Arch Linux — Etapa 01

A Etapa 01 define a infraestrutura comum, mas **não produz ainda um payload de runtime instalável**. Por isso nenhum `PKGBUILD` fictício ou pacote vazio é gerado.

Quando uma etapa futura introduzir um componente instalável, o cenário Arch deverá:

1. consumir o manifesto versionado da própria etapa;
2. compilar/testar como usuário normal;
3. montar payload somente em staging/`$pkgdir`;
4. gerar pacote nativo através de `makepkg`;
5. validar o pacote produzido;
6. instalar exatamente esse artefato através de `pacman -U`;
7. registrar nome, versão, caminho e SHA-256 no handoff.

O `PKGBUILD` é um formato nativo do ecossistema Arch e pode usar a sintaxe exigida por `makepkg`; isso não altera a obrigação de os wrappers Nexxus permanecerem `#!/bin/sh` 100% POSIX.
