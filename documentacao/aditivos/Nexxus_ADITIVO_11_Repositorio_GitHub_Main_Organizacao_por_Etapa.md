# NEXXUS — DOCUMENTO ADITIVO 11

## Repositório GitHub Canônico, Publicação Direta na `main` e Organização por Etapa

**Status:** APROVADO  
**Natureza:** DOCUMENTAÇÃO ADITIVA  
**Data:** 16/08/2026

### PRINCÍPIO CENTRAL

O repositório GitHub canônico do Projeto Nexxus passa a ser:

**https://github.com/mintonogueira/nexxus-de**

Toda etapa/conversa de construção deverá possuir pasta própria e pertinente dentro do repositório. Ao concluir uma etapa, todo material versionável e todos os artefatos de entrega definidos pelas fontes do Nexxus deverão ser publicados no repositório canônico e o estado final da etapa deverá estar presente na branch `main`.

O agente de desenvolvimento está **expressamente autorizado pelo usuário a realizar publicações, commits e atualizações necessárias na branch `main`** desse repositório para cumprir as entregas do Projeto Nexxus, respeitando as regras de segurança, preservação de histórico, isolamento por etapa e não alteração do resultado final.

---

## 1. Objetivo

Formalizar o endereço do repositório GitHub canônico do Nexxus, estabelecer a `main` como branch canônica de publicação, definir organização obrigatória por pasta para cada conversa/etapa e registrar a autorização explícita do usuário para publicação real do trabalho no repositório.

Este aditivo complementa a Documentação Completa do Projeto Nexxus e os Aditivos 01 a 10. Ele não reescreve silenciosamente documentos anteriores. Ele **substitui apenas os pontos anteriormente marcados como EM ABERTO no Aditivo 06 relativos à URL do repositório e à autorização operacional de publicação**, preservando suas demais regras de segurança e rastreabilidade.

## 2. Repositório canônico

Fica definido como repositório oficial e canônico do Projeto Nexxus:

`https://github.com/mintonogueira/nexxus-de`

- Proprietário: `mintonogueira`
- Repositório: `nexxus-de`
- Branch canônica de publicação: `main`
- Todo código, script, teste, manifesto, configuração, documentação técnica, handoff e demais arquivos versionáveis pertencentes ao Nexxus deverão ser destinados a esse repositório.
- O agente não deverá solicitar novamente o link enquanto esta definição permanecer vigente.

## 3. `main` como branch canônica obrigatória

A branch `main` passa a representar o estado oficial publicado do projeto.

- Ao concluir cada etapa, o estado final validado daquela etapa deve estar na `main`.
- Nenhuma etapa poderá ser declarada **PUBLICADA** se seu conteúdo final permanecer apenas localmente ou somente em outra branch.
- O fluxo normal deverá privilegiar commits seguros diretamente na `main`, pois o usuário autorizou expressamente essa operação.
- Branches temporárias não constituem destino final e não podem substituir a publicação na `main`.
- Se alguma ferramenta ou situação técnica exigir branch temporária, todo conteúdo aprovado deverá chegar à `main` antes do encerramento da etapa.
- Não realizar `force-push`, reescrita destrutiva de histórico ou exclusão de conteúdo remoto desconhecido sem autorização específica do usuário.

## 4. Uma pasta por conversa/etapa

Cada conversa de construção do Nexxus deverá corresponder a uma pasta própria, claramente identificável e exclusiva daquela etapa.

Padrão recomendado:

```text
etapas/
  etapa-01-core-fundacao/
  etapa-02-<nome-da-etapa>/
  etapa-03-<nome-da-etapa>/
  ...
```

Cada pasta poderá conter apenas o material pertinente à própria etapa, por exemplo:

```text
etapas/etapa-NN-<slug>/
  README.md
  src/
  scripts/
  packaging/
  tests/
  docs/
  dist/
  entrega/
```

A estrutura interna exata poderá variar conforme a natureza do módulo, mas a separação por etapa/conversa é obrigatória.

## 5. Organização do código

- Cada módulo/código deverá permanecer na pasta pertinente à etapa que o construiu ou mantém conforme o contrato do projeto.
- Código não deverá ser despejado na raiz do repositório sem organização.
- Arquivos compartilhados do workspace poderão permanecer em áreas comuns quando tecnicamente necessários, desde que a origem e a responsabilidade da etapa sejam rastreáveis.
- Mudanças compartilhadas devem ser registradas no handoff da etapa.
- Nenhuma pasta de uma etapa autoriza desenvolver silenciosamente outro módulo.

## 6. Publicação obrigatória ao concluir a etapa

Quando uma etapa atingir seus critérios de aceite, deverá ocorrer, antes do fechamento formal:

1. finalizar código, scripts, testes e documentação da própria etapa;
2. executar as validações exigidas;
3. atualizar a versão pertinente;
4. gerar os pacotes/artefatos exigidos pelos aditivos aplicáveis;
5. gerar e validar o arquivo compactado de entrega exigido pelo Aditivo 09;
6. organizar todo o conteúdo na pasta da etapa;
7. verificar ausência de segredos, caches e temporários;
8. versionar as alterações;
9. publicar efetivamente na branch `main` do repositório canônico;
10. verificar o commit publicado;
11. registrar no handoff o commit/hash real e os caminhos publicados;
12. somente então classificar `STATUS_GITHUB = PUBLICADO`.

## 7. Artefatos para download na `main`

A entrega compactada da etapa e, quando pertinente e tecnicamente viável, os pacotes binários de entrega deverão ser mantidos em área claramente identificável da pasta da etapa, por exemplo:

```text
etapas/etapa-NN-<slug>/entrega/
```

O objetivo é permitir que o usuário encontre e baixe facilmente a entrega diretamente a partir do estado publicado na `main`.

A regra não autoriza ultrapassar silenciosamente limites técnicos rígidos do GitHub. Se um artefato não puder ser armazenado na `main` por limitação objetiva da plataforma, a etapa deverá registrar o bloqueio e retornar ao usuário para decisão, sem substituir silenciosamente o método de entrega.

## 8. Preparação da pasta da etapa subsequente

Ao concluir e publicar uma etapa, deverá ser criada na `main` a pasta pertinente à **etapa subsequente já definida pela coordenação**.

Como Git não versiona diretórios vazios, a pasta deverá conter um arquivo mínimo de inicialização, preferencialmente `README.md`, contendo somente:

- identificação da próxima etapa;
- status `PRONTA_PARA_INICIAR`;
- referência ao handoff da etapa anterior;
- indicação de que o desenvolvimento ocorrerá em nova conversa.

A criação dessa pasta é apenas preparação estrutural do repositório. **Não significa iniciar o desenvolvimento da etapa subsequente na conversa atual**, preservando integralmente o Aditivo 04.

Se a próxima etapa ainda não estiver definida, nenhuma pasta deverá ser inventada. O ponto permanece `EM ABERTO` até a coordenação definir a etapa seguinte.

## 9. Autorização expressa de publicação

O usuário concede ao agente de desenvolvimento autorização operacional expressa para:

- criar arquivos e pastas pertinentes ao Nexxus no repositório canônico;
- criar commits;
- atualizar a branch `main` por avanço normal e não destrutivo;
- publicar o código e os artefatos versionáveis das etapas concluídas;
- publicar documentação, handoffs, scripts POSIX, manifests, testes e arquivos de entrega pertinentes;
- preparar a pasta da próxima etapa após o encerramento da etapa atual.

Essa autorização permanece limitada ao repositório:

`https://github.com/mintonogueira/nexxus-de`

e ao Projeto Nexxus.

## 10. Limites da autorização

A autorização de publicação **não** autoriza:

- alterar requisitos do Nexxus;
- publicar material de outros projetos;
- enviar segredos, tokens, credenciais ou chaves privadas;
- apagar histórico remoto desconhecido;
- executar `force-push`;
- excluir branches, tags ou arquivos preexistentes sem necessidade segura e autorização pertinente;
- substituir silenciosamente trabalho remoto divergente;
- iniciar código de outra etapa na conversa atual;
- declarar sucesso de publicação se a operação não tiver sido realmente executada.

## 11. Relação com o Aditivo 06

Este Aditivo 11 fecha os pontos operacionais anteriormente em aberto no Aditivo 06:

| Campo | Definição anterior | Nova definição |
|---|---|---|
| REPOSITORIO_GITHUB | URL em aberto | `https://github.com/mintonogueira/nexxus-de` |
| BRANCH CANÔNICA | Não congelada | `main` |
| AUTORIZAÇÃO DE PUBLICAÇÃO | Dependente de acesso/autorização | EXPRESSAMENTE CONCEDIDA pelo usuário |
| ORGANIZAÇÃO POR ETAPA | Genérica | Uma pasta pertinente por conversa/etapa |
| ETAPA SUBSEQUENTE | Apenas indicada no handoff | Pasta estrutural criada após conclusão, sem iniciar código |

As regras de segurança do Aditivo 06 permanecem vigentes.

## 12. Relação com o Aditivo 09

O arquivo `.tar.gz` obrigatório de cada etapa continua sendo parte da entrega material. Quando tecnicamente viável dentro dos limites do GitHub, ele deverá ser organizado na pasta `entrega/` da etapa e publicado na `main`, mantendo coerência com o commit e o handoff.

## 13. Handoff ampliado

A partir deste aditivo, o fechamento de cada etapa deverá registrar:

- `REPOSITORIO_GITHUB`: `https://github.com/mintonogueira/nexxus-de`
- `BRANCH`: `main`
- `PASTA_DA_ETAPA`
- `COMMIT_MAIN`
- `STATUS_GITHUB`
- `ARQUIVOS_PUBLICADOS`
- `ARTEFATOS_PARA_DOWNLOAD`
- `ARQUIVO_COMPACTADO`
- `SHA256`
- `PASTA_PROXIMA_ETAPA`
- `STATUS_PROXIMA_ETAPA`
- `PENDENCIAS_DE_PUBLICACAO`, se houver

## 14. Critérios de aceite

Uma etapa somente poderá ser declarada publicada quando:

- sua pasta pertinente existir no repositório;
- o código e os scripts da etapa estiverem organizados;
- os artefatos obrigatórios estiverem entregues conforme as regras aplicáveis;
- o commit real estiver presente na `main`;
- o handoff registrar o hash correspondente;
- não houver segredos ou arquivos indevidos publicados;
- a pasta estrutural da próxima etapa estiver criada quando a próxima etapa já estiver definida;
- nenhuma implementação da próxima etapa tiver sido iniciada na conversa anterior.

## 15. Registro da decisão aditiva

| Campo | Definição |
|---|---|
| Status | APROVADO |
| Data | 16/08/2026 |
| Repositório canônico | `https://github.com/mintonogueira/nexxus-de` |
| Branch canônica | `main` |
| Publicação por etapa | Obrigatória após conclusão/validação |
| Organização | Uma pasta pertinente por conversa/etapa |
| Próxima etapa | Criar pasta estrutural após encerramento quando já definida |
| Autorização | Usuário autoriza expressamente publicação e atualização normal da `main` |
| Limite | Sem operações destrutivas, sem segredos, sem alteração do resultado final e sem invasão de outra etapa |

## 16. Regra final

```text
1 CONVERSA = 1 ETAPA = 1 PASTA PERTINENTE

ETAPA CONCLUÍDA
  -> VALIDAR
  -> EMPACOTAR
  -> ORGANIZAR A PASTA
  -> VERSIONAR
  -> PUBLICAR NA MAIN
  -> VERIFICAR COMMIT
  -> REGISTRAR HANDOFF
  -> CRIAR PASTA ESTRUTURAL DA PRÓXIMA ETAPA
  -> ENCERRAR A CONVERSA

REPOSITÓRIO CANÔNICO:
https://github.com/mintonogueira/nexxus-de

BRANCH CANÔNICA:
main

AUTORIZAÇÃO:
PUBLICAÇÃO NO REPOSITÓRIO E NA MAIN EXPRESSAMENTE AUTORIZADA PELO USUÁRIO.
```

A publicação real no GitHub passa a fazer parte do fechamento normal das etapas do Nexxus. O repositório canônico deverá refletir, na `main`, o estado oficialmente concluído e entregue do projeto.
