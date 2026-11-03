# Curriculum Database

Self-hosted [Dolt](https://www.dolthub.com/): exam content and its whole history. MySQL wire protocol.

|                       |                                                                                                   |
| --------------------- | ------------------------------------------------------------------------------------------------- |
| `compose.yaml`        | Pinned `dolthub/dolt-sql-server` image (tag + digest), one volume, `curriculum-db` alias.         |
| `config/server.yaml`  | Server config. Explicit data/config paths on the volume, auto-GC off, file import disabled.       |
| `scripts/server.sh`   | Entrypoint. Disables metrics and version checks, checks the root password, starts the server.     |
| `migrations/`         | `NNNN_name.sql`, contiguous from `0001`. Applied by hand for now: [schema](#changing-the-schema). |
| `fixtures/sample.sql` | Synthetic single-choice exam. No `curriculum_issues`.                                             |
| Host port             | `127.0.0.1:13306`                                                                                 |
| Database              | `curriculum`. Branches `dev`, `stg`, `prd`.                                                       |

Only `root` exists. App roles for `eDd` and `aAPI` are not set up yet: [roles](#roles-not-set-up).

## Setup

```bash
cd databases/curriculum
cp .env.example .env   # CDB_ROOT_PASSWORD=$(openssl rand -hex 32)
docker compose up -d --wait
```

`scripts/server.sh` replaces the image's entrypoint, so restarts never create users, run init files or migrate. `sql-server` creates `root` from `CDB_ROOT_PASSWORD` only when no privileges file exists. To rotate, run `ALTER USER 'root'@'%' IDENTIFIED BY '<new>'`, then update `CDB_ROOT_PASSWORD` and recreate the container. If the variable is changed without the `ALTER USER`, the healthcheck fails.

SQL shell, through the container's `dolt` CLI. `--use-db` takes `curriculum/<branch>` or `curriculum/<commit hash>`:

```bash
cdb() { docker compose exec -T curriculum-db sh -c 'DOLT_CLI_PASSWORD="$DOLT_ROOT_PASSWORD" exec dolt --host 127.0.0.1 --port 3306 --user root --no-tls "$@"' cdb "$@"; }
cdb sql -q 'SELECT DOLT_VERSION()'
```

## Config

`dolt_transaction_commit` is disabled as this produces a noisy history.

## Dolt

`dolt_checkout()` on one pooled connection does **not** apply to the next connection from the pool. The recommended way to handle this is to rollback a dropped connection with:

```sql
BEGIN; call dolt_checkout(<branch>); ... COMMIT;
```

Dolt does not support `SELECT FOR UPDATE`.

## Migrations

The migration runner works much the same as with the Postgres databases - `sqlx` runs the migrations based on `.sql` files.

On top of the migration, the runner also needs to run `CALL dolt_commit("-Am", "<message>")` after applying the changes. This makes each migration a discrete commit in Dolt's history which makes diffing and rollback easy.

Migrations should only be applied on `dev`, then merge `dev` into other content branches - never run migrations independently on two branches.

`_sqlx_migrations` table is in versioned path, because ignoring it would lead to sqlx not noticing applied migrations.

### GC and Deletion

1. Create a backup
2. As root, run `CALL DOLT_GC()`

Never use `docker compose down -v`, `docker volume prune` or hard resets on real data.
