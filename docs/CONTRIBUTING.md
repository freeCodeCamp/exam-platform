# Contributing

## First Time Setup

### Prerequisites

| Tool                                                              | Version                                   | For                                      |
| ----------------------------------------------------------------- | ----------------------------------------- | ---------------------------------------- |
| [Rust](https://rustup.rs/)                                        | stable, `>= 1.95` (`rust-toolchain.toml`) | aAPI, eDd server, eDb, mDb, dApp backend |
| [Bun](https://bun.sh/)                                            | `1.4.2` (CI)                              | wApp, eDd client, dApp frontend          |
| [Docker](https://docs.docker.com/engine/install/)                 | with Compose                              | eDb, mDb, images                         |
| [`sqlx-cli`](https://crates.io/crates/sqlx-cli)                   | `0.9.0`                                   | eDb, mDb migrations, `.sqlx/`            |
| [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)  | Linux: webkit2gtk-4.1, libsoup-3          | dApp only                                |
| [`mdbook`](https://rust-lang.github.io/mdBook/), `mdbook-mermaid` | `0.5.4`, `0.17.1` (CI)                    | docs only                                |

```bash
# For working with database stuffs
cargo install sqlx-cli --version 0.9.0 --locked --no-default-features --features rustls,postgres,sqlx-toml
# For working on docs/
cargo install mdbook@0.5.4 mdbook-mermaid@0.17.1 --locked # optional
```

### Clone

```bash
git clone https://github.com/freeCodeCamp/exam-platform.git
cd exam-platform
bun install
# Hook to catch if offline db changes are not committed
git config core.hooksPath .githooks
docker network create platform
```

### Environment

Copy every example, then fill blanks:

```bash
for d in apps/auth-api apps/examiner-dashboard apps/web-app apps/desktop-app databases/examiner databases/moderation; do
  cp -n "$d/.env.example" "$d/.env"
done
```

`.env` is read automatically only by:

| Reads `.env` automatically            | Does not                                  |
| ------------------------------------- | ----------------------------------------- |
| Bun (wApp, eDd client, dApp)          | `cargo run`, `cargo test`, `cargo clippy` |
| Vite (dApp frontend)                  | rust-analyzer                             |
| `docker compose` (from its directory) |                                           |
| `sqlx-cli` (from its directory)       |                                           |

Rust apps read only the process environment. Export the app's `.env` into the shell before `cargo`:

```bash
set -a; source apps/auth-api/.env; set +a
cargo run -p auth-api
```

Or use [`direnv`](https://direnv.net/): `echo dotenv > .envrc && direnv allow` in each app directory, then run `cargo` from there. `.envrc` is not in `.gitignore`; do not commit it.

| File                           | Set                                                                                                                                                                                             |
| ------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `databases/examiner/.env`      | `EDB_PASSWORD`, and same password in `EDB_DATABASE_URL`                                                                                                                                         |
| `databases/moderation/.env`    | `MDB_PASSWORD`, `MDB_EDD_PASSWORD`, and `MDB_PASSWORD` in `MDB_DATABASE_URL`                                                                                                                    |
| `apps/auth-api/.env`           | `MDB_DATABASE_URL` (same as above, owner `aapi`)                                                                                                                                                |
| `apps/examiner-dashboard/.env` | `EDB_DATABASE_URL` (same as above), `MDB_DATABASE_URL` (as `edd`, with `MDB_EDD_PASSWORD`), `COOKIE_KEY` (`openssl rand -hex 64`), `GOOGLE_OAUTH_ID`, `GOOGLE_OAUTH_SECRET` ([Auth](./auth.md)) |
| Others                         | Defaults work for `ENVIRONMENT=development`. `SENTRY_DSN` stays empty                                                                                                                           |

All variables: [Environment Variables](./deployment/environment-variables.md).

### eDb

eDd runs migrations on startup, and the `query!` macros need eDb (or `.sqlx/`) to compile.

```bash
cd databases/examiner
docker compose up -d --wait
set -a; source .env; set +a
cargo sqlx migrate run
```

### mDb

aAPI runs migrations on startup; eDd only checks them, so start aAPI first. Same steps from `databases/moderation/`. Roles: [databases/moderation](../databases/moderation/README.md#roles).

### Editor

VS Code: install recommended extensions (`.vscode/extensions.json`). Settings run clippy on save, set `ENVIRONMENT=development` for rust-analyzer, and link both Cargo workspaces.

## Development

### Run

| App        | Command                                           | Port              |
| ---------- | ------------------------------------------------- | ----------------- |
| aAPI       | `cargo run -p auth-api`                           | `127.0.0.1:13001` |
| eDd server | `cargo run -p examiner-dashboard`                 | `127.0.0.1:13002` |
| eDd client | `bun --filter examiner-dashboard-client dev`      | `127.0.0.1:13005` |
| wApp       | `bun --filter web-app dev`                        | `127.0.0.1:13003` |
| dApp       | `bun run dev:desktop`                             | `localhost:1420`  |
| eDb        | `docker compose up -d` in `databases/examiner/`   | `127.0.0.1:15432` |
| mDb        | `docker compose up -d` in `databases/moderation/` | `127.0.0.1:15433` |
| Docs       | `mdbook watch` (or `mdbook serve`)                | `localhost:3000`  |

`cargo run` rows need their `.env` exported first ([Environment](#environment)). `bun run dev` starts wApp and eDd client together. eDd client proxies `/api` to eDd server. wApp and dApp call aAPI at `AUTH_API_URL`.

Images: `docker compose -f apps/<app>/compose.yaml up --build`. Build context is repo root.

### Workspaces

| Workspace | Manifest                      | Members                                               |
| --------- | ----------------------------- | ----------------------------------------------------- |
| Cargo     | `Cargo.toml`                  | `auth-api`, `examiner-dashboard`, `edb`, `libs/*`     |
| Cargo     | `apps/desktop-app/Cargo.toml` | dApp backend. Separate so servers never resolve Tauri |
| Bun       | `package.json`                | `web-app`, `examiner-dashboard-client`, `desktop-app` |

Shared versions live in `package.json` catalogs: reference with `catalog:` or `catalog:astro`, do not pin in members.

### Check

CI (`.github/workflows/ci.yml`) runs only jobs whose paths changed. Run the same locally before pushing:

```bash
# Root Cargo workspace (eDb running; source databases/examiner/.env for EDB_DATABASE_URL)
cargo fmt --all --check
SQLX_OFFLINE=true cargo clippy --workspace --all-targets --locked
cargo clippy -p runtime --no-default-features --all-targets --locked
DATABASE_URL=$EDB_DATABASE_URL cargo test --workspace --locked

# dApp backend
cargo fmt --all --check --manifest-path apps/desktop-app/Cargo.toml
cargo clippy --workspace --all-targets --locked --manifest-path apps/desktop-app/Cargo.toml
cargo test --workspace --locked --manifest-path apps/desktop-app/Cargo.toml

# Bun
bun run check
bun run build
```

Clippy `all` is `deny`. Format Rust with `rustfmt` (`rustfmt.toml`), eDd client with `bun --filter examiner-dashboard-client fmt` (oxfmt).

### Database Changes

eDb schema, migrations and queries live in crate `edb` (`databases/examiner/`); mDb in crate `mdb` (`databases/moderation/`, `MDB_DATABASE_URL`). Steps below use eDb; mDb is the same from its directory, plus its [grant rules](../databases/moderation/README.md#changing-the-schema). Run every step from that directory with the database up (`docker compose up -d --wait`).

`sqlx-cli` reads `EDB_DATABASE_URL` from its `.env`; `cargo` does not, so export it first.

1. Create a reversible migration pair:

```bash
cargo sqlx migrate add -r -s <name> # migrations/NNNN_<name>.{up,down}.sql
```

2. Write `up` and `down` SQL. `down` is for local development only; production never runs it.
3. Apply it to local eDb:

```bash
cargo sqlx migrate run
```

To iterate on uncommitted migration: `cargo sqlx migrate revert`, make edits, then run `cargo sqlx migrate run`.

4. Add or change queries in `src/` (one module per table group, each an `impl Database` block of `query!`/`query_as!`). `cargo check -p edb` validates them against local eDb.
5. Test with `#[sqlx::test]`. It reads `DATABASE_URL`, not `sqlx.toml`, and creates a migrated database per test:

```bash
DATABASE_URL=$EDB_DATABASE_URL cargo test -p edb
```

6. Refresh offline query data:

```bash
cargo sqlx prepare
```

Pre-commit hook (`git config core.hooksPath .githooks`) runs this and stages `.sqlx/` on commit when a database's files are staged. It never blocks the commit: if it warns with `cargo sqlx prepare failed; .sqlx/ may be stale`, manually run step 6.

7. Commit migration, queries and `.sqlx/` together. Docker and CI build with `SQLX_OFFLINE=true`. CI's clippy fails on a stale `.sqlx/`.

### Commits and PRs

[Conventional Commits](https://www.conventionalcommits.org/): release-please reads them to version each app ([Releases](./releases.md)).

```
<type>(<scope>): <summary>
```

| Part  | Values                                                                                    |
| ----- | ----------------------------------------------------------------------------------------- |
| type  | `feat`, `fix`, `docs`, `chore`, `dev`. `feat` and `fix` trigger release; `chore` does not |
| scope | App or area: `aAPI`, `eDd`, `eDb`, `wApp`, `dApp`, `docs`, `.github`. Omit when repo-wide |

Follow [Conventions](./conventions/README.md).
