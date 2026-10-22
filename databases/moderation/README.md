# Moderation Database (mDb)

Postgres. All candidate data: identity, attempts, responses, evidence, moderation outcomes. Two users: aAPI owns the schema, eDd reads candidate data and writes moderation outcomes ([auth](../../docs/auth.md)). Crate `mdb` (root Cargo workspace) holds the schema, migrations and queries for both.

|               |                                                                                  |
| ------------- | -------------------------------------------------------------------------------- |
| `migrations/` | `NNNN_name.{up,down}.sql`. Embedded; run by `Database::connect` on aAPI startup. |
| `src/`        | One module per table group, each an `impl Database` block of checked `query!`s.  |
| `initdb/`     | Creates the `edd` login role on an empty volume.                                 |
| `.sqlx/`      | Offline query metadata. Committed; Docker and CI build from it.                  |
| `sqlx.toml`   | Points the macros and sqlx-cli at `MDB_DATABASE_URL`, not `DATABASE_URL`.        |
| Host port     | `127.0.0.1:15433`                                                                |

## Roles

| Role      | Login | Used by | Privileges                                                                                    |
| --------- | ----- | ------- | --------------------------------------------------------------------------------------------- |
| `aapi`    | yes   | aAPI    | Owner. Runs migrations. Hosted Postgres: database owner with `CREATEROLE`.                    |
| `mdb_edd` | no    | -       | `SELECT` on every table (default privileges), writes granted per table. Created by `0001`.    |
| `edd`     | yes   | eDd     | Member of `mdb_edd`. `initdb/edd.sh` creates it on an empty volume; otherwise create by hand. |

```sql
CREATE ROLE edd LOGIN PASSWORD '...' IN ROLE mdb_edd;
```

eDd connects with `Database::connect_checked`: it never migrates, and exits unless every migration it embeds is applied with a matching checksum.

## Setup

Needs `sqlx-cli` 0.9: `cargo install sqlx-cli --version 0.9.0 --locked --no-default-features --features rustls,postgres,sqlx-toml`.

```bash
cp .env.example .env   # set MDB_PASSWORD and MDB_EDD_PASSWORD, and add MDB_PASSWORD in MDB_DATABASE_URL
docker compose up -d --wait
set -a; source .env; set +a
cargo sqlx migrate run
git config core.hooksPath .githooks   # pre-commit refreshes and stages .sqlx/
```

## Changing the schema

1. `cargo sqlx migrate add -r -s <name>` from this directory.
2. Write the `up` and `down` SQL. `down` is for local development only; production never runs it.
3. A table eDd writes: grant it in the same migration, e.g. `GRANT INSERT, UPDATE ON moderation_outcomes TO mdb_edd`. Reads are granted by default.
4. Change the queries, then commit with mDb running: the pre-commit hook runs `cargo sqlx prepare` and stages `.sqlx/`. It never blocks the commit, so check its warning; CI's offline clippy fails on a stale `.sqlx/`.
5. Tests: `DATABASE_URL=$MDB_DATABASE_URL cargo test -p mdb`. `#[sqlx::test]` reads `DATABASE_URL`, not `sqlx.toml`, and creates a migrated database per test. Tests check `mdb_edd` with `SET LOCAL ROLE`.

Rules:

- Only add, release, then remove ([releases](../../docs/releases.md)), across **both** apps: drop or rename only once neither the running nor the `previous` image of aAPI or eDd uses the object.
- aAPI deploys migrations. An eDd release that needs a new migration deploys after the aAPI release that ships it; deployed earlier, eDd fails its startup check and the deploy rolls back.
- Never edit an applied migration: sqlx checksums each one, and both aAPI and eDd refuse to start.
