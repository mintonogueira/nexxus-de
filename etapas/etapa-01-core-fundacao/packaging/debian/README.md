# Contrato de empacotamento Debian — Etapa 01

A Etapa 01 define a infraestrutura comum, mas **não produz ainda um payload de runtime instalável**. Nenhum `.deb` vazio é criado para simular empacotamento concluído.

Quando uma etapa futura introduzir um componente instalável, o cenário Debian deverá:

1. separar dependências de build e runtime no manifesto da etapa;
2. compilar/testar como usuário normal;
3. construir a árvore do pacote em staging, sem escrita direta no host;
4. gerar um `.deb` real com metadados Debian coerentes;
5. validar estrutura, metadados e dependências;
6. instalar exatamente o `.deb` recém-gerado através do mecanismo nativo apropriado;
7. registrar nome, versão, caminho e SHA-256 no handoff.
