# Examiner Database

Hosted (eventually) Postgres database.

|                                        |                                                                                                                 |
| -------------------------------------- | --------------------------------------------------------------------------------------------------------------- |
| `initdb/`                              | Bootstrap: roles, schemas, passwords. Docker runs it on an empty volume; tests run `10-bootstrap.sql` per test. |
| `migrations/`                          | `NNNN_name.{up,down}.sql`, one global order for both apps. Embedded in `edb-migrate`.                           |
| `src/bin/edb-migrate.rs`               | Applies pending migrations as `edb_migrator`, then exits. Never run by the apps.                                |
| `src/edd/`, `src/aapi/`, `src/shared/` | One module per table group, each an `impl Database` block of checked `query!`s.                                 |
| `.sqlx/`                               | Offline query metadata. Committed; Docker and CI build from it.                                                 |
| `sqlx.toml`                            | `EDB_MIGRATOR_URL` for the macros and `cargo sqlx`; migration history in `edb._sqlx_migrations`.                |
| Host port                              | `127.0.0.1:15432`                                                                                               |

## Roles

| Role           | Owns                                   | Privileges                                                                            | Used by                          |
| -------------- | -------------------------------------- | ------------------------------------------------------------------------------------- | -------------------------------- |
| `postgres`     | -                                      | Superuser                                                                             | Bootstrap and tests. Never apps. |
| `edb_migrator` | schemas `edb`, `edd`, `aapi`, `shared` | Everything in them; no `CREATE` on the database                                       | `edb-migrate`, `cargo sqlx`      |
| `edd`          | -                                      | Read/write `edd.users`, `edd.sessions`; read `aapi.candidates`; per-table in `shared` | eDd                              |
| `aapi`         | -                                      | Read/write `aapi.candidates`; per-table in `shared`. Nothing in `edd`                 | `aAPI`                           |

Bootstrap creates the roles and schemas, and revokes `TEMP` from `PUBLIC`. Migrations create every table and grant the app roles access to it explicitly: no default privileges, so each new table is a deliberate decision. Neither app role can read `edb` - the migration history. Queries should name the schema (e.g. `edd.users`), and should never rely on unqualified `search_path`.

`edb-migrate` (env `EDB_MIGRATOR_URL`) holds a Postgres advisory lock, so concurrent runs from both backends' releases apply each migration once. It waits at most 60s for a lock (`edb::LOCK_TIMEOUT`) and 300s overall, then exits non-zero. It verifies the checksums of the migrations it embeds, and ignores applied migrations it does not: an older release runs against a newer schema. Each migration runs in a transaction: failed ones roll back, earlier ones stay applied.

Deploys run the `edb-migrate` bin from the image being deployed, before replacing the app, as the one-shot compose service `migrate`. The apps never migrate or check schema versions.

For hosted Postgres: as an admin role, run `initdb/10-bootstrap.sql`, then `ALTER ROLE <role> PASSWORD '...'` for `edb_migrator`, `edd`, and `aapi`.

## Schemas

Schemas are organizational. They are not ownership or access boundaries. `edb_migrator` owns every table, and grants decide what each app reads and writes.

| Schema   | Holds                                                       |
| -------- | ----------------------------------------------------------- |
| `edb`    | Migration history. No app access.                           |
| `edd`    | Tables only `eDd` writes. `aAPI` may get `SELECT`.          |
| `aapi`   | Tables only `aAPI` writes. `eDd` may get `SELECT`.          |
| `shared` | Tables both apps write. Each role's grants still per table. |

Place a table alongside its writers, not its readers: one writer goes in that app's schema, two go in `shared`. Moving a table between schemas later is a rename, so it follows the add, release, remove rule.

Bootstrap is idempotent and creates `shared` like the others: `CREATE SCHEMA IF NOT EXISTS shared AUTHORIZATION edb_migrator`. It cannot be a migration, as `edb_migrator` has no `CREATE` on the database (DDL boundary).

Foreign keys may cross schemas (e.g. `shared.x.candidate_id REFERENCES aapi.candidates (id)`). `edb_migrator` owns both ends, so no `REFERENCES` grant is needed, and RI checks run as the owner, so the writing app needs no `SELECT` on the referenced table. A cross-schema FK still couples the apps:

- Deliberately choose `ON DELETE`. With the default, the referenced app's deletes start failing once the other app's rows point at them, and its running image may not expect that. Adding the FK must work with both running images and their rollbacks.
- On a populated table, add it as `NOT VALID`, then `VALIDATE CONSTRAINT` in a later migration: validation scans the table, and the add locks both tables.
- Dropping the referenced table or column needs the FK dropped first (in an earlier release).

## Setup

Needs `sqlx-cli` 0.9: `cargo install sqlx-cli --version 0.9.0 --locked --no-default-features --features rustls,postgres,sqlx-toml`.

```bash
cp .env.example .env   # set the four passwords, and the same in the two URLs
docker compose up -d --wait
set -a; source .env; set +a
cargo run -p edb --bin edb-migrate
git config core.hooksPath .githooks   # pre-commit refreshes and stages .sqlx/
```

Start either app. `initdb/` should **only** be run on an empty volume. Changing or resetting: `docker compose down -v`, then the steps above.

## Changing the schema

From this directory:

1. `cargo sqlx migrate add -r -s <name>`.
2. Write the `up` and `down` SQL, naming the schema. Pick the schema by who writes the table ([schemas](#schemas)). `down` is for local development only; production never runs it.
3. Grant each app role exactly what it needs on the new table, in the same migration, e.g. `GRANT SELECT, INSERT, UPDATE, DELETE ON edd.moderation_outcomes TO edd` and `GRANT SELECT ON edd.moderation_outcomes TO aapi`. A new schema also needs `GRANT USAGE`.
4. `cargo run -p edb --bin edb-migrate` (or `cargo sqlx migrate run`). Iterate on an uncommitted migration with `cargo sqlx migrate revert`.
5. Change the queries, then run `cargo sqlx prepare` with eDb running: builds are offline (`.cargo/config.toml`), so a new or changed query fails to compile until `.sqlx/` has it. The pre-commit hook also runs it and stages `.sqlx/`. It never blocks the commit, so check its warning; CI's offline clippy fails on a stale `.sqlx/`.
6. Tests: `DATABASE_URL=$EDB_ADMIN_URL cargo test -p edb`. Use `#[sqlx::test(migrations = false)]` and call `testing::setup` first: it bootstraps, then migrates as `edb_migrator`. Query through `testing::database_as(&pool, Role::Edd)` to test as the app does; test denials with `SET LOCAL ROLE`.

Rules:

- Only add, release, then remove: drop or rename only once no running or `previous` image of `eDd` or `aAPI` uses the object. Each migration must work with both the running images and their rollback images. Nothing checks this.
- Either backend's release applies every migration in its build. Both apps embed the crate, so both release on any change here; whichever deploys first migrates.
- Code that needs a new migration can ship with it: the deploy migrates before starting the app. The other app's running image must still work with the new schema.
- Never edit an applied migration: sqlx checksums each one, and `edb-migrate` fails, so no backend deploys.
- A failed migration blocks every backend deploy until a release fixes it. Schema changes are never rolled back automatically.
