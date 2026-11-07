# Curriculum Database

Self-hosted [Dolt](https://www.dolthub.com/): exam content and its whole history. MySQL wire protocol.

|                          |                                                                                                        |
| ------------------------ | ------------------------------------------------------------------------------------------------------ |
| `compose.yaml`           | Pinned `dolthub/dolt-sql-server` image (tag + digest), one volume, `curriculum-db` alias.              |
| `config/server.yaml`     | Server config. Explicit data/config paths on the volume.                                               |
| `scripts/server.sh`      | Entrypoint. Disables metrics and version checks, checks the root password, starts the server.          |
| `migrations/`            | `NNNN_name.sql`, contiguous from `0001`. Applied to `dev` by `cdb-migrate`: [migrations](#migrations). |
| `src/bin/cdb-migrate.rs` | Migrator. Ships in eDd's image only; eDd's deploys run it (`migrate-cdb` service).                     |
| Host port                | `127.0.0.1:13306`                                                                                      |

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

### Once-off

Create the database with `dev` branch, then migrate it. `cargo test -p cdb` uses the same URL, and creates and drops a database per test:

```bash
cdb sql -q 'CREATE DATABASE curriculum'
set -a; . ./.env; set +a
export CDB_MIGRATOR_URL="mysql://root:$CDB_ROOT_PASSWORD@127.0.0.1:13306/curriculum"
cargo run -p cdb --bin cdb-migrate
```

## Config

`dolt_transaction_commit` is disabled as this produces a noisy history.

## Dolt

A session's branch is session state. `dolt_checkout()` stays on a pooled connection for the next borrower, and `ROLLBACK` does not undo it: `BEGIN; CALL dolt_checkout('x'); ROLLBACK;` leaves `ACTIVE_BRANCH()` on `x`. Never rely on the branch a pooled connection happens to be on. Either:

a. Start every transaction by naming its branch. `USE` a revision database, which reads, writes, and `dolt_commit`s on that branch without touching checkout state:

```sql
BEGIN; USE `curriculum/<branch>`; ... COMMIT;
```

b. Give each branch its own pool, whose connections run that `USE` on connect (sqlx `after_connect`) and never call `dolt_checkout`.

Table names can also be branch-qualified: `` `curriculum/prd`.exams ``. Only `cdb-migrate` calls `dolt_checkout`, on a connection it opens and closes itself.

Dolt does not support `SELECT FOR UPDATE`.

## Objects

The bytes of question images and audio live in the curriculum bucket, not in Dolt. `objects` holds one row per file, keyed by SHA-256, with its object key (`sha256/<sha256>.<ext>`), sniffed content type, byte size and original file name. `question_objects` orders a question's objects and holds their alt text. Rows carry keys, never URLs, so old snapshots keep resolving after a provider or CDN change.

Uploading is not committing. eDd records each upload in eDb (`edd.objects`, see [eDd](./examiner-dashboard.md#question-objects)). Not built yet: when an examiner commits content that references an object, eDd is to copy its row into `objects` in that same Dolt commit.

## Migrations

`cdb-migrate` (`CDB_MIGRATOR_URL`) applies `migrations/` with sqlx, as `edb-migrate` does for eDb, with these differences for Dolt:

Dolt-specific notes:

- Only `dev` receives the migration
  - Branches get the schema when `dev` is merged into them. Never independently run migrations on two branches.
- One Dolt commit per migration
  - Message `cdb-migrate: NNNN <description>`, author `cdb-migrate`.
  - Each commit holds the schema change and its `_sqlx_migrations` row, so diffing, reverting, and cherry-picking a migration is one commit.
- Applied on a scratch branch
  - Each run, under a `GET_LOCK` named for the database:
  1. Recreates branch `cdb-migrate` from `dev`'s HEAD and checks it out.
  2. Applies each pending migration there, then `CALL DOLT_COMMIT('-A', ...)`.
  3. Merges `cdb-migrate` into `dev`: a fast-forward, or a merge commit (`cdb-migrate: merge NNNN..MMMM`) if `dev` moved meanwhile. A conflict aborts the merge and fails the run.
  4. Deletes `cdb-migrate`.
- `_sqlx_migrations` table is versioned
  - Not in `dolt_ignore`.
  - Otherwise, `sqlx` would not see applied migrations on other branches, nor after a merge.

The scratch branch is needed because:

- Dolt commits the SQL transaction on DDL, so a migration is never atomic.
- `dolt_commit -A` on `dev` would sweep in whatever uncommitted writes other sessions left in `dev`'s working set.
- `dolt_commit` inside an open transaction can commit a stale snapshot, so it always runs after the migration's transaction.

On the scratch branch, a failed or interrupted run leaves `dev` untouched. The next run recreates the branch and starts over. A failed run leaves `cdb-migrate` behind for inspection, with `--use-db curriculum/cdb-migrate`.

Only eDd runs `cdb-migrate`, before each of its deploys, after eDb's `migrate`. aAPI only reads cDb, from branches whose schema changes only on a merge from `dev`. Running it there gains nothing and would give aAPI's stack DDL credentials.

TODO: merging `dev` into `stg`/`prd` also brings unpromoted content and `manifest` rows. Cherry-picking the per-migration commits is an option.

### GC and Deletion

Automatic GC is on (`config/server.yaml`). GC runs in the background as garbage builds up, including from each `cdb-migrate` run's deleted scratch branch.

Never disable automatic GC without a replacement. With it off, every `CALL DOLT_GC()` breaks all open connections, fails in-flight queries, and leaves the calling connection unusable. Apps would then need to retry and reconnect, and GC would need maintenance windows.

Never use `docker compose down -v`, `docker volume prune` or hard resets on real data.
