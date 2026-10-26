# Releases

Exams are on-demand - there can be no deploy freeze.

Every app that builds is its own release-please component.

Consider release service to perform smart releases - `rApp`

## Apps

| App                  | Ships as                                                     | Version files release-please bumps                     |
| -------------------- | ------------------------------------------------------------ | ------------------------------------------------------ |
| `auth-api`           | Image → Komodo stack `exam-platform-auth-api`                | `Cargo.toml`, root `Cargo.lock`                        |
| `examiner-dashboard` | Image (client + server) → `exam-platform-examiner-dashboard` | `Cargo.toml`, `client/package.json`, root `Cargo.lock` |
| `web-app`            | Image (Caddy serving `dist/`) → `exam-platform-web-app`      | `package.json`                                         |
| `desktop-app`        | Tauri bundles: the release job is a stub                     | `package.json`, `backend/Cargo.toml`, `Cargo.lock`     |

## Workflow

1. Merges to `main` cause `release.yml` release-please PRs for affected applications.
2. Merged release-please PRs call `release-container.yml` to build the apps, pushes them to the registry, and calls `deploy.yml`.
3. `deploy.yml` awaits staging GH environment approval, then tells Komodo to run deploy action. For aAPI and eDd, the Action first runs the image's `edb-migrate`, and deploys only if it succeeds.

## Rollback

Stacks read `compose.yaml` from `main`, so a rollback runs the old image with the current compose file: see [Rollback across compose changes](#rollback-across-compose-changes).

Rollback reverts the app image only, never eDb's schema: the old image runs against the newer schema. Migrations keep it backward compatible: drop or rename only once no running or `previous` image uses the object.

### Through GitHub

Go to `Actions → Deploy → Run workflow`, with the app and `previous`, or with any digest from a release's **Image** line. `previous` deploys `_IMAGE_PREVIOUS`, and on success swaps it with `_IMAGE_HEALTHY`.

### Without GitHub

aAPI and eDd: set `EXAM_PLATFORM_<APP>_MIGRATE_IMAGE` to the digest, run the stack's `migrate` service, and continue only if it exits 0. Then set `EXAM_PLATFORM_<APP>_IMAGE` in Komodo, and deploy the stack. Once it is healthy, set `_IMAGE_HEALTHY` to the same digest: the next failed deploy rolls back to it. Komodo runs Actions from the UI without arguments, so the Action cannot do this.

### Rollback Across Compose Changes

A deploy pairs two inputs from different places: the image digest from the Komodo Variable, and `compose.yaml` from `main` at the moment of the deploy. Therefore, a rollback runs the old image with the newest compose file. That works while the compose file changes only in ways the old image tolerates.

- Safe: adding environment variables (an image ignores those it does not read), labels, logging or resource limits; changing the restart policy or a published host port.
- Unsafe: renaming or removing a variable it still reads, or changing what one means; changing `LISTEN_ADDR`, the container port, `WEB_DIR` or another path baked into the image; dropping a network alias that another app, or Caddy, still calls.

Keep compose changes backward compatible for one release, as with database migrations: add, release, then remove in a later release. Commit them as `fix` or `feat` so they ship with the app's next release. A `chore` change sits on `main` until the app's next deploy, and that may be a rollback.

To roll back across an incompatible compose change:

1. Preferred: revert the compose change on `main`, then deploy `previous`.
2. Emergency: in Komodo, set the stack's Commit to the commit of the release being restored (GitHub tag `<app>-v<version>`), then deploy `previous`. Its `compose.yaml` matches the image again. Clear the Commit once `main` is fixed. Until then the sync reports the stack as changed, and running the sync reverts it to `main`.
