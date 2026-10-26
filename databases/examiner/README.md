# Examiner Database (eDb)

Postgres, shared by eDd and aAPI. One schema per app (`edd`, `aapi`) for organization; role `edb_migrator` owns all of them, and the apps connect as runtime roles that cannot change the schema ([auth](../../docs/auth.md)). Crate `edb` (root Cargo workspace) holds the schema, migrations, queries and the `edb-migrate` binary.

|                         |                                                                                                                   |
| ----------------------- | ----------------------------------------------------------------------------------------------------------------- |
| `initdb/`               | Bootstrap: roles, schemas, passwords. Docker runs it on an empty volume; tests run `10-bootstrap.sql` per test.   |
| `migrations/`           | `NNNN_name.{up,down}.sql`, one global order for both apps. Embedded in `edb-migrate`.                             |
| `src/bin/edb-migrate.rs` | Applies pending migrations as `edb_migrator`, then exits. Never run by the apps.                                 |
| `src/edd/`, `src/aapi/` | One module per table group, each an `impl Database` block of checked `query!`s.                                   |
| `.sqlx/`                | Offline query metadata. Committed; Docker and CI build from it.                                                   |
| `sqlx.toml`             | `EDB_MIGRATOR_URL` for the macros and `cargo sqlx`; migration history in `edb._sqlx_migrations`.                  |
| Host port               | `127.0.0.1:15432`                                                                                                 |

## Roles

| Role           | Owns                          | Privileges                                                                             | Used by                            |
| -------------- | ----------------------------- | -------------------------------------------------------------------------------------- | ---------------------------------- |
| `postgres`     | -                             | Superuser                                                                              | Bootstrap, tests. Never apps.      |
| `edb_migrator` | schemas `edb`, `edd`, `aapi`  | Everything in them; no `CREATE` on the database                                        | `edb-migrate`, `cargo sqlx`        |
| `edd`          | -                             | Read/write `edd.users`, `edd.sessions`; read `aapi.candidates`                         | eDd                                |
| `aapi`         | -                             | Read/write `aapi.candidates`. Nothing in `edd`                                         | aAPI                               |

Bootstrap creates the roles and schemas, and revokes `TEMP` from `PUBLIC`. Migrations create every table and grant the app roles access to it explicitly: no default privileges, so each new table is a deliberate decision. Neither app role can read `edb`, the migration history. Queries always name the schema (`edd.users`), never rely on `search_path`.

`edb-migrate` (env `EDB_MIGRATOR_URL`) holds a Postgres advisory lock, so concurrent runs from both backends' releases apply each migration once. It waits at most 60s for a lock (`edb::LOCK_TIMEOUT`) and 300s overall, then exits non-zero. It verifies the checksums of the migrations it embeds, and ignores applied migrations it does not: an older release runs against a newer schema. Each migration runs in a transaction; a failed one rolls back, earlier ones stay applied.

Deploys run it from the image being deployed, before replacing the app, as the one-shot compose service `migrate`. The apps never migrate or check schema versions.

Hosted Postgres, or an existing volume: as an admin role, run `initdb/10-bootstrap.sql`, then `ALTER ROLE <role> PASSWORD '...'` for `edb_migrator`, `edd` and `aapi`.

## Setup

Needs `sqlx-cli` 0.9: `cargo install sqlx-cli --version 0.9.0 --locked --no-default-features --features rustls,postgres,sqlx-toml`.

```bash
cp .env.example .env   # set the four passwords, and the same in the two URLs
docker compose up -d --wait
set -a; source .env; set +a
# Offline: the query macros cannot check against an unmigrated database.
SQLX_OFFLINE=true cargo run -p edb --bin edb-migrate
git config core.hooksPath .githooks   # pre-commit refreshes and stages .sqlx/
```

Then start either app, in any order. `initdb/` runs on an empty volume only. After changing it, or to reset: `docker compose down -v`, then the steps above.

## Changing the schema

From this directory:

1. `cargo sqlx migrate add -r -s <name>`.
2. Write the `up` and `down` SQL, naming the schema. `down` is for local development only; production never runs it.
3. Grant each app role exactly what it needs on the new table, in the same migration, e.g. `GRANT SELECT, INSERT, UPDATE, DELETE ON edd.moderation_outcomes TO edd` and `GRANT SELECT ON edd.moderation_outcomes TO aapi`. A new schema also needs `GRANT USAGE`.
4. `cargo run -p edb --bin edb-migrate` (or `cargo sqlx migrate run`). Iterate on an uncommitted migration with `cargo sqlx migrate revert`.
5. Change the queries, then commit with eDb running: the pre-commit hook runs `cargo sqlx prepare` and stages `.sqlx/`. It never blocks the commit, so check its warning; CI's offline clippy fails on a stale `.sqlx/`.
6. Tests: `DATABASE_URL=$EDB_ADMIN_URL cargo test -p edb`. Use `#[sqlx::test(migrations = false)]` and call `testing::setup` first: it bootstraps, then migrates as `edb_migrator`. Query through `testing::database_as(&pool, Role::Edd)` to test as the app does; test denials with `SET LOCAL ROLE`.

Rules:

- Only add, release, then remove ([releases](../../docs/releases.md)): drop or rename only once no running or `previous` image of eDd or aAPI uses the object. Each migration must work with both the running images and their rollback images. Nothing checks this.
- Either backend's release applies every migration in its build. Both apps embed the crate, so both release on any change here; whichever deploys first migrates.
- Code that needs a new migration can ship with it: the deploy migrates before starting the app. The other app's running image must still work with the new schema.
- Never edit an applied migration: sqlx checksums each one, and `edb-migrate` fails, so no backend deploys.
- A failed migration blocks every backend deploy until a release fixes it. Schema changes are never rolled back automatically.
