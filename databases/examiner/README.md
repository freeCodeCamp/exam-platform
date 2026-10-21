# Examiner Database (eDb)

Postgres. eDd is its only user. Crate `edb` (root Cargo workspace) holds the schema, migrations and queries.

|               |                                                                                    |
| ------------- | ---------------------------------------------------------------------------------- |
| `migrations/` | `NNNN_name.{up,down}.sql`. Embedded and run by `Database::connect` on eDd startup. |
| `src/`        | One module per table group, each an `impl Database` block of checked `query!`s.    |
| `.sqlx/`      | Offline query metadata. Committed; Docker and CI build from it.                    |
| `sqlx.toml`   | Points the macros and sqlx-cli at `EDB_DATABASE_URL`, not `DATABASE_URL`.          |
| Host port     | `127.0.0.1:15432`                                                                  |

## Setup

Needs `sqlx-cli` 0.9: `cargo install sqlx-cli --version 0.9.0 --locked --no-default-features --features rustls,postgres,sqlx-toml`.

```bash
cp .env.example .env   # set EDB_PASSWORD, and add it in EDB_DATABASE_URL
docker compose up -d --wait
set -a; source .env; set +a
cargo sqlx migrate run
git config core.hooksPath .githooks   # pre-commit refreshes and stages .sqlx/
```

## Changing the schema

1. `cargo sqlx migrate add -r -s <name>` from this directory.
2. Write the `up` and `down` SQL. `down` is for local development only; production never runs it.
3. Change the queries, then commit with eDb running: the pre-commit hook runs `cargo sqlx prepare` and stages `.sqlx/`. It never blocks the commit, so check its warning; CI's offline clippy fails on a stale `.sqlx/`.
4. Tests: `DATABASE_URL=$EDB_DATABASE_URL cargo test -p edb`. `#[sqlx::test]` reads `DATABASE_URL`, not `sqlx.toml`, and creates a migrated database per test.

Rules:

- Only add, release, then remove ([releases](../../docs/releases.md)). A rolled back eDd runs against the newer schema, so every migration must keep the previous release working. The migrator skips applied migrations it does not know, so a rollback starts; nothing checks that the old code still works with the newer schema.
- Never edit an applied migration: sqlx checksums each one and eDd refuses to start.
