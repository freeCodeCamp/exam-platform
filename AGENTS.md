This is a monorepo for the following:

| Location                     | Short Name | Long Name            | Description                                                                                |
| ---------------------------- | ---------- | -------------------- | ------------------------------------------------------------------------------------------ |
| `./apps/auth-api/`           | `aAPI`     | Auth API             | HTTP REST API for `wApp` and `dApp`                                                        |
| `./apps/desktop-app/`        | `dApp`     | Desktop App          | Tauri desktop app candidates take the exams on                                             |
| `./apps/examiner-dashboard/` | `eDd`      | Examiner Dashboard   | Fullstack web app for staff to create and edit exams, and moderate and grade exam attempts |
| `./apps/web-app/`            | `wApp`     | Web App              | Static web app where candidates register accounts and exam attempts                        |
| `./databases/curriculum/`    | `cDb`      | Curriculum Database  | Dolt database and configuration with SDK for exam content                                  |
| `./databases/examiner/`      | `eDb`      | Examiner Database    | Postgres configuration and SDK for `eDd` and `aAPI` data                                   |
| `./libs/object-storage/`     | -          | Object Storage crate | Rust crate for the S3-compatible storage solutions for non-text data                       |
| `./libs/runtime/`            | -          | Runtime crate        | Rust crate for common rust APIs in server setup                                            |

## LLM/Agentic Conventions

Never write code comments. All code comments should be human-written.

Do not touch Markdown content in `./docs/`. The `check-docs` skill can be used to bring up changes that need to be made.
