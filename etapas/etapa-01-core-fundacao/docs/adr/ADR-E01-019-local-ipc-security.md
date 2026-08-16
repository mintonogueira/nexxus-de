# ADR-E01-019 — Endurecimento do IPC local

**Status:** aceito — 2026-08-16

## Decisão

O endpoint Unix da fundação só é criado abaixo de diretório privado pertencente ao UID corrente. Symlinks, arquivos não-socket e sockets de outro usuário são rejeitados. Um socket existente do mesmo usuário só é removido quando uma conexão retorna `ConnectionRefused`, caracterizando stale endpoint. O socket criado usa modo `0600`.

No `Drop`, o path só é removido quando `device+inode` ainda identificam o socket criado pela própria instância.

## Razão

Evita unlink arbitrário, substituição por symlink e remoção de endpoint que tenha sido trocado durante a vida do processo, mantendo o IPC local dentro do princípio de menor privilégio.
