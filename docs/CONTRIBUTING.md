# Contributing

## Quick Start

Ensure [Bun](https://bun.sh/) and [Docker](https://docs.docker.com/engine/install) are installed.

```bash
git clone https://github.com/freeCodeCamp/exam-platform.git
cd exam-platform
bun run dev all
```

## First Time Setup

### Prerequisites

| Tool                                                              | Version                                   | For                                 |
| ----------------------------------------------------------------- | ----------------------------------------- | ----------------------------------- |
| [Rust](https://rustup.rs/)                                        | stable, `>= 1.95` (`rust-toolchain.toml`) | aAPI, eDd server, eDb, dApp backend |
| [Bun](https://bun.sh/)                                            | `1.4.2` (CI)                              | wApp, eDd client, dApp frontend     |
| [Docker](https://docs.docker.com/engine/install/)                 | with Compose                              | eDb, images                         |
| [`sqlx-cli`](https://crates.io/crates/sqlx-cli)                   | `0.9.0`                                   | eDb migrations, `.sqlx/`            |
| [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)  | Linux: webkit2gtk-4.1, libsoup-3          | dApp only                           |
| [`mdbook`](https://rust-lang.github.io/mdBook/), `mdbook-mermaid` | `0.5.4`, `0.17.1` (CI)                    | docs only                           |

```bash
# For working with database stuffs
cargo install sqlx-cli --version 0.9.0 --locked --no-default-features --features rustls,postgres,sqlx-toml
# For working on docs/
cargo install mdbook@0.5.4 mdbook-mermaid@0.17.1 --locked # optional
```

|                       | Rust | Bun | Docker | Tauri | mdbook |
| --------------------- | :--: | :-: | :----: | :---: | :----: |
| `auth-api`            |  ✅  |  X  |   X    |   X   |   X    |
| `curriculum-database` |  ✅  |  X  |   ✅   |   X   |   X    |
| `desktop-app`         |  ✅  | ✅  |   X    |  ✅   |   X    |
| `examiner-dashboard`  |  ✅  | ✅  |   X    |   X   |   X    |
| `examiner-database`   |  ✅  |  X  |   ✅   |   X   |   X    |
| `web-app`             |  X   | ✅  |   X    |   X   |   X    |
| `docs/`               |  X   |  X  |   X    |   X   |   ✅   |
| `libs/`               |  ✅  |  X  |   X    |   X   |   X    |

### Clone

```bash
git clone https://github.com/freeCodeCamp/exam-platform.git
cd exam-platform
bun install
docker network create platform
```

`bun install` also installs the Husky pre-commit hook.

### Environment

Use [`direnv`](https://direnv.net/): `echo dotenv > .envrc && direnv allow` in each app directory, then run `cargo` from there. `.envrc` is not in `.gitignore`; do not commit it.

| File                           | Set                                                                                                                                                         |
| ------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `databases/examiner/.env`      | `EDB_PASSWORD`, `EDB_MIGRATOR_PASSWORD`, `EDB_EDD_PASSWORD`, `EDB_AAPI_PASSWORD`, and the same in `EDB_ADMIN_URL`, `EDB_MIGRATOR_URL`                       |
| `apps/auth-api/.env`           | `EDB_DATABASE_URL` (as `aapi`, with `EDB_AAPI_PASSWORD`)                                                                                                    |
| `apps/examiner-dashboard/.env` | `EDB_DATABASE_URL` (as `edd`, with `EDB_EDD_PASSWORD`), `COOKIE_KEY` (`openssl rand -hex 64`), `GOOGLE_OAUTH_ID`, `GOOGLE_OAUTH_SECRET` ([Auth](./auth.md)) |
| Others                         | Defaults work for `ENVIRONMENT=development`. `SENTRY_DSN` stays empty                                                                                       |

All variables: [Environment Variables](./deployment/environment-variables.md).

### eDb

```bash
cd databases/examiner
docker compose up -d --wait
set -a; source .env; set +a
cargo run -p edb --bin edb-migrate
```

Run `edb-migrate` again after pulling new migrations. Reset with `docker compose down -v`.

### Editor

VS Code: install recommended extensions (`.vscode/extensions.json`). Settings run clippy on save, set `ENVIRONMENT=development` for rust-analyzer, and link both Cargo workspaces.

## Development

### Run

#### Slices (Docker)

| Slice  | Starts                                                              |
| ------ | ------------------------------------------------------------------- |
| `edb`  | eDb, migrated                                                       |
| `cdb`  | cDb, database `curriculum` created and migrated                     |
| `s3`   | SeaweedFS on `127.0.0.1:18333`, bucket `curriculum`                 |
| `edd`  | eDd server with built client on `127.0.0.1:13002`, plus `edb`, `s3` |
| `aapi` | aAPI on `127.0.0.1:13001`, plus `edb`                               |
| `wapp` | wApp on `127.0.0.1:13003`, plus `aapi`                              |
| `all`  | Everything                                                          |

```bash
bun run dev edd          # combine: bun run dev edd wapp
bun run dev -d edd       # background
bun run dev logs edd     # follow logs
bun run dev ps
bun run dev down         # stop, keep data
bun run dev reset        # stop, delete data volumes
bun run dev compose ...  # any docker compose command on compose.dev.yaml
```

#### Native

| App        | Command                                         | Port              |
| ---------- | ----------------------------------------------- | ----------------- |
| aAPI       | `cargo run -p auth-api`                         | `127.0.0.1:13001` |
| eDd server | `cargo run -p examiner-dashboard`               | `127.0.0.1:13002` |
| eDd client | `bun --filter examiner-dashboard-client dev`    | `127.0.0.1:13005` |
| wApp       | `bun --filter web-app dev`                      | `127.0.0.1:13003` |
| dApp       | `bun run dev:desktop`                           | `localhost:1420`  |
| eDb        | `docker compose up -d` in `databases/examiner/` | `127.0.0.1:15432` |
| Docs       | `mdbook watch` (or `mdbook serve`)              | `localhost:3000`  |

### Workspaces

| Workspace | Manifest                      | Members                                               |
| --------- | ----------------------------- | ----------------------------------------------------- |
| Cargo     | `Cargo.toml`                  | `auth-api`, `examiner-dashboard`, `edb`, `libs/*`     |
| Cargo     | `apps/desktop-app/Cargo.toml` | dApp backend. Separate so servers never resolve Tauri |
| Bun       | `package.json`                | `web-app`, `examiner-dashboard-client`, `desktop-app` |

Shared versions live in `package.json` catalogs: reference with `catalog:` or `catalog:astro`, do not pin in members.

### Tests

```bash
bun run dev -d edb cdb
DATABASE_URL="$EDB_ADMIN_URL" cargo test
bun run test
```

### Database Changes

Never edit committed, applied migrations.

Start the database(s):

```bash
bun run dev -d edb cdb
```

#### `eDb`

Create a reversible migration:

```bash
cargo sqlx migrate add -r -s <name>
```

Write generated `up` and `down` SQL, including schema names and app-role grants. Update queries and tests, then apply, refresh offline query metadata, and test:

```bash
cargo run -p edb --bin edb-migrate
cargo sqlx prepare
DATABASE_URL="$EDB_ADMIN_URL" cargo test -p edb
```

For local, uncommitted migration iteration, use `cargo sqlx migrate revert` before editing and reapplying.

#### `cDb`

Create a forward-only migration:

```bash
cargo sqlx migrate add -s <name>
```

Write generated SQL and update queries/tests, then apply and test:

```bash
cargo run -p cdb --bin cdb-migrate
cargo test -p cdb
```

Use `cdb-migrate`, not `cargo sqlx migrate run`: it applies migrations on scratch branch, creates Dolt commits, then merges into `dev`. Other branches receive schema changes through merges from `dev`.

`CDB_MIGRATOR_URL` must name the database, not a branch revision.

#### Commit

Commit migrations, queries, and tests together; include `.sqlx/` changes for eDb. Follow [database conventions](./conventions/databases.md).

### Commits and PRs

[Conventional Commits](https://www.conventionalcommits.org/): release-please reads them to version each app ([Releases](./releases.md)).

```
<type>(<scope>): <summary>
```

| Part  | Values                                                                                             |
| ----- | -------------------------------------------------------------------------------------------------- |
| type  | `feat`, `fix`, `docs`, `chore`, `dev`, `refactor`. `feat` and `fix` trigger release; others do not |
| scope | App or area: `aAPI`, `eDd`, `cDb`, `eDb`, `wApp`, `dApp`, `docs`, `.github`. Omit when repo-wide   |

Follow [Database](./conventions/databases.md) and [Rust](./conventions/rust.md) conventions.
