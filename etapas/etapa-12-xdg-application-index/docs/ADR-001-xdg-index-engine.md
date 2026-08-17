# ADR-001 — Motor do XDG Application Index

- **Etapa:** 12 — XDG Application Index
- **Status:** APROVADO COMO DECISAO TECNICA INTERNA
- **Data:** 2026-08-17

## Problema

A Etapa 12 precisa produzir um catálogo XDG correto, dinâmico e reutilizável sem depender de gerenciadores de pacote e sem introduzir execução por shell ao interpretar `Exec`.

## Decisão

1. `freedesktop-desktop-entry` 0.8.1 é usado somente para decodificação Desktop Entry/localização. A feature `gettext` é desabilitada para evitar uma dependência de sistema desnecessária; a seleção de locale é resolvida pelo próprio módulo conforme Desktop Entry 1.5.
2. A precedência, Desktop File ID, mascaramento por `Hidden` e complementação de raízes Flatpak são controlados pelo Nexxus para manter o contrato explícito e testável.
3. `notify` 8.2.x fornece o backend de eventos de filesystem. Em Linux o backend recomendado usa inotify. Eventos são debounceados e provocam rescan completo determinístico; instalação de aplicações é rara e essa estratégia reduz complexidade/caches incrementais inconsistentes.
4. `Exec` é revalidado pelo Nexxus e transformado em template de argv. O módulo nunca executa `sh -c`, nunca concatena uma linha de shell e rejeita field codes desconhecidos ou posições proibidas.
5. `nexxus-assets` da Etapa 08 é a dependência visual usada somente para fallback. Um `Icon=` externo é preservado sem recoloração. Caminhos absolutos permanecem caminhos; nomes simbólicos permanecem nomes XDG para a política de tema do consumidor.
6. `NoDisplay` não apaga a entrada do catálogo: ela permanece consultável, mas é excluída das views visíveis. `Hidden=true` mascara o Desktop File ID inteiro, inclusive cópias de menor precedência.
7. Flatpak é integrado por seus diretórios exportados XDG. Nenhum `flatpak list`, pacman ou APT participa da descoberta.

## Dependências avaliadas

### freedesktop-desktop-entry 0.8.1

- licença upstream: MPL-2.0;
- escopo: parser Desktop Entry e acesso tipado às chaves necessárias;
- `gettext` opcional foi removido do feature set da Etapa 12;
- evita implementar manualmente escaping/localized values do formato completo.

### notify 8.2.x

- backend recomendado por plataforma; Linux usa inotify;
- não é tratado como fonte de verdade: eventos apenas disparam um rescan XDG;
- falhas do watcher são emitidas como diagnóstico de serviço, não derrubam snapshots já válidos.

## Segurança

- arquivos `.desktop` possuem limite de 2 MiB por leitura;
- symlinks de diretório não são percorridos, evitando ciclos;
- um ID de maior precedência é reservado antes do parse, impedindo fallback inesperado para uma cópia de sistema quando o override do usuário está inválido;
- `Exec` expande arquivos/URLs em elementos argv separados e nunca reinterpreta o conteúdo expandido;
- código Rust desta etapa proíbe `unsafe`.

## Fontes primárias consultadas

- Desktop Entry Specification 1.5 — Freedesktop;
- XDG Base Directory Specification 0.8 — Freedesktop;
- Desktop Menu Specification / Category Registry — Freedesktop;
- Icon Theme Specification 0.13 — Freedesktop;
- Flatpak documentation — Desktop Integration / Conventions;
- documentação upstream do `notify` 8.2.0 e código/documentação do `freedesktop-desktop-entry` 0.8.1.

## Impacto

Nenhum contrato de etapa anterior é modificado. Etapas 13, 14 e 16 poderão consumir snapshots/eventos sem conhecer pacman, APT, Flatpak CLI ou detalhes do watcher.
