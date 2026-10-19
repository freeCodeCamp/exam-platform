# Environment Variables

Environment variables are _a question mark_

Questions

- many `.env` files, duplication of things like `SUCH_SUCH_DOMAIN`
- many `.env` files, but root `.env` for any common vars
- `exam-platform/env/common.env`
- separate variables vs secrets (different files)
- variables in `.env`, secrets in a secret store (cloud)
- build vs runtime

## Variables

"Where Set" lists the place that supplies the value. `.env` is the app's own `.env` (copy `.env.example`). Komodo means the stack `environment` in `deploy/exam-platform.toml`.

### `apps/auth-api`

| Name                        | Required            | Default         | Where Set                                            | Description                                                                          |
| --------------------------- | ------------------- | --------------- | ---------------------------------------------------- | ------------------------------------------------------------------------------------ |
| `ENVIRONMENT`               | yes                 |                 | `.env`, Komodo (`staging`)                           | `development`, `staging` or `production`. Release images refuse to start without it. |
| `SENTRY_DSN`                | outside development | empty           | `.env`, Komodo (`EXAM_PLATFORM_AUTH_API_SENTRY_DSN`) | Sentry project DSN.                                                                  |
| `SENTRY_TRACES_SAMPLE_RATE` | no                  | `0.2`           | `.env`                                               | Sentry trace sampling rate.                                                          |
| `RUST_LOG`                  | no                  | `info`          | `.env`                                               | Filters what reaches stdout (development) and Sentry.                                |
| `AAPI_PORT`                 | no                  | `13001`         | `.env`                                               | Host port on `127.0.0.1`. Container always listens on `13001`.                       |
| `LISTEN_ADDR`               | no                  | `0.0.0.0:13001` | `compose.yaml`, `Dockerfile`                         | Bind address. Do not change: baked into image.                                       |
| `PLATFORM_NETWORK`          | no                  | `platform`      | `.env`                                               | Shared external Docker network name.                                                 |
| `IMAGE`                     | no                  | empty           | `.env`, Komodo (`EXAM_PLATFORM_AUTH_API_IMAGE`)      | Release image to run. Empty builds from source. Komodo sets on deploys.              |

### `apps/examiner-dashboard`

| Name                        | Required                   | Default                              | Where Set                                                      | Description                                                                                                                                                                        |
| --------------------------- | -------------------------- | ------------------------------------ | -------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `ENVIRONMENT`               | yes                        |                                      | `.env`, Komodo (`staging`)                                     | `development`, `staging` or `production`.                                                                                                                                          |
| `SENTRY_DSN`                | outside development        | empty                                | `.env`, Komodo (`EXAM_PLATFORM_EXAMINER_DASHBOARD_SENTRY_DSN`) | Sentry project DSN.                                                                                                                                                                |
| `SENTRY_TRACES_SAMPLE_RATE` | no                         | `0.2`                                | `.env`                                                         | Sentry trace sampling rate.                                                                                                                                                        |
| `RUST_LOG`                  | no                         | `info`                               | `.env`                                                         | Filters what reaches stdout (development) and Sentry.                                                                                                                              |
| `EDD_PORT`                  | no                         | `13002`                              | `.env`                                                         | Host port on `127.0.0.1`. Container always listens on `13002`.                                                                                                                     |
| `LISTEN_ADDR`               | no                         | `0.0.0.0:13002`                      | `compose.yaml`, `Dockerfile`                                   | Bind address. Do not change: baked into image.                                                                                                                                     |
| `WEB_DIR`                   | no                         | `dist` natively; `/app/web` in image | Process environment, `compose.yaml`, `Dockerfile`              | Static directory relative to working directory. Client builds to `apps/examiner-dashboard/dist`; default works when run from the app directory. No startup check for `index.html`. |
| `GOOGLE_OAUTH_ID`           | unless mock defaults apply | none                                 | Process environment, `.env` for Compose, Komodo                | Google OAuth web client ID; must not be empty.                                                                                                                                     |
| `GOOGLE_OAUTH_SECRET`       | unless mock defaults apply | none                                 | Process environment, `.env` for Compose, Komodo                | OAuth client secret; must not be empty.                                                                                                                                            |
| `GOOGLE_REDIRECT_URL`       | unless mock defaults apply | none                                 | Process environment, `.env` for Compose, Komodo                | Valid callback URL registered in Google Cloud: public HTTPS origin plus `/auth/google/callback`.                                                                                   |
| `COOKIE_KEY`                | unless mock defaults apply | none                                 | Process environment, `.env` for Compose, Komodo                | Private-cookie key, at least 64 bytes. Generate with `openssl rand -hex 64`; keep secret and stable. Rotation invalidates cookies.                                                 |
| `MOCK_AUTH`                 | no                         | `false`                              | Process environment                                            | Debug-only mock auth. Release builds reject `true`; Compose does not forward it. Defaults for OAuth/key fields apply only when unset, not empty.                                   |
| `PLATFORM_NETWORK`          | no                         | `platform`                           | `.env`                                                         | Shared external Docker network name.                                                                                                                                               |
| `IMAGE`                     | no                         | empty                                | `.env`, Komodo (`EXAM_PLATFORM_EXAMINER_DASHBOARD_IMAGE`)      | Release image to run. Empty builds from source. Komodo sets on deploys.                                                                                                            |

Native dashboard execution needs exported environment variables; copying `.env` alone does not load them. See [dashboard setup](../../apps/examiner-dashboard/README.md#setup). Mock mode defaults to test credentials/key and `http://127.0.0.1:13002/auth/google/callback`. Persistence remains stubbed for both real and mock auth.

### `apps/web-app`

| Name               | Required | Default                  | Where Set                                      | Description                                                                              |
| ------------------ | -------- | ------------------------ | ---------------------------------------------- | ---------------------------------------------------------------------------------------- |
| `AUTH_API_URL`     | yes      | `http://127.0.0.1:13001` | `.env`, Komodo                                 | Runtime. Auth API origin the browser calls, served as `/config.js`. Default in dev only. |
| `WAPP_PORT`        | no       | `13003`                  | `.env`                                         | Host port on `127.0.0.1`. Container always listens on `13003`.                           |
| `PLATFORM_NETWORK` | no       | `platform`               | `.env`                                         | Shared external Docker network name.                                                     |
| `IMAGE`            | no       | empty                    | `.env`, Komodo (`EXAM_PLATFORM_WEB_APP_IMAGE`) | Release image to run. Empty builds from source. Komodo sets on deploys.                  |

### `apps/desktop-app`

| Name           | Required                         | Default                                 | Where Set            | Description                                                 |
| -------------- | -------------------------------- | --------------------------------------- | -------------------- | ----------------------------------------------------------- |
| `ENVIRONMENT`  | no                               | `production`                            | `.env`, compile time | `development`, `staging` or `production`. Read with `env!`. |
| `SENTRY_DSN`   | unless `ENVIRONMENT=development` | empty                                   | `.env`, compile time | Sentry project DSN. Empty disables Sentry in development.   |
| `AUTH_API_URL` | unless `ENVIRONMENT=development` | `http://127.0.0.1:13001` in development | `.env`               | Auth API origin.                                            |

### `deploy/`

Set in `deploy/.env` on the Droplet. `bootstrap.sh` copies `.env.example`, then fills each empty secret once with `openssl rand -hex 32`.

| Name                          | Required | Default                               | Where Set                 | Description                                                     |
| ----------------------------- | -------- | ------------------------------------- | ------------------------- | --------------------------------------------------------------- |
| `KOMODO_DOMAIN`               | yes      | `exam-platform.freecodecamp.dev`      | `deploy/.env`             | Komodo public name. DNS points at the Droplet.                  |
| `AUTH_API_DOMAIN`             | yes      | `auth-api.freecodecamp.dev`           | `deploy/.env`             | Caddy host for auth API.                                        |
| `EXAMINER_DASHBOARD_DOMAIN`   | yes      | `examiner-dashboard.freecodecamp.dev` | `deploy/.env`             | Caddy host for examiner dashboard.                              |
| `WEB_APP_DOMAIN`              | yes      | `web-app.freecodecamp.dev`            | `deploy/.env`             | Caddy host for web app.                                         |
| `KOMODO_INIT_ADMIN_USERNAME`  | yes      | `admin`                               | `deploy/.env`             | First admin, created on first start only.                       |
| `KOMODO_INIT_ADMIN_PASSWORD`  | yes      | generated                             | `deploy/.env` (bootstrap) | First admin password. Change after login.                       |
| `KOMODO_DATABASE_USERNAME`    | yes      | `komodo`                              | `deploy/.env`             | MongoDB root user, created on first start only.                 |
| `KOMODO_DATABASE_PASSWORD`    | yes      | generated                             | `deploy/.env` (bootstrap) | MongoDB root password.                                          |
| `KOMODO_JWT_SECRET`           | yes      | generated                             | `deploy/.env` (bootstrap) | Signs login sessions. Rotating logs everyone out.               |
| `KOMODO_WEBHOOK_SECRET`       | yes      | generated                             | `deploy/.env` (bootstrap) | Authenticates webhooks. None configured, but Core requires one. |
| `KOMODO_GOOGLE_OAUTH_ENABLED` | no       | `false`                               | `deploy/.env`             | Enables Google login for Komodo.                                |
| `KOMODO_GOOGLE_OAUTH_ID`      | no       | empty                                 | `deploy/.env`             | Google Cloud OAuth client ID. Set by hand, restart Core.        |
| `KOMODO_GOOGLE_OAUTH_SECRET`  | no       | empty                                 | `deploy/.env`             | Google Cloud OAuth client secret. Set by hand, restart Core.    |

### Komodo variables

Per app, `EXAM_PLATFORM_<APP>_`, with `<APP>` one of `AUTH_API`, `EXAMINER_DASHBOARD`, `WEB_APP`.

| Name                                          | Required | Default | Where Set                        | Description                                                        |
| --------------------------------------------- | -------- | ------- | -------------------------------- | ------------------------------------------------------------------ |
| `EXAM_PLATFORM_<APP>_IMAGE`                   | yes      |         | Komodo, written by deploy Action | Image the stack runs. Set before each deploy, so may be unhealthy. |
| `EXAM_PLATFORM_<APP>_IMAGE_HEALTHY`           | no       |         | Komodo, written by deploy Action | Last image confirmed healthy. Rollback target.                     |
| `EXAM_PLATFORM_<APP>_IMAGE_PREVIOUS`          | no       |         | Komodo, written by deploy Action | Healthy image before that. Target for `previous`.                  |
| `EXAM_PLATFORM_AUTH_API_SENTRY_DSN`           | yes      |         | Komodo                           | Passed to auth-api as `SENTRY_DSN`.                                |
| `EXAM_PLATFORM_EXAMINER_DASHBOARD_SENTRY_DSN` | yes      |         | Komodo                           | Passed to examiner-dashboard as `SENTRY_DSN`.                      |

Dashboard also requires these Komodo Variables (no defaults):

| Name                                                   | Description                                           |
| ------------------------------------------------------ | ----------------------------------------------------- |
| `EXAM_PLATFORM_EXAMINER_DASHBOARD_GOOGLE_OAUTH_ID`     | Passed as `GOOGLE_OAUTH_ID`.                          |
| `EXAM_PLATFORM_EXAMINER_DASHBOARD_GOOGLE_OAUTH_SECRET` | Secret; passed as `GOOGLE_OAUTH_SECRET`.              |
| `EXAM_PLATFORM_EXAMINER_DASHBOARD_GOOGLE_REDIRECT_URL` | Registered callback; passed as `GOOGLE_REDIRECT_URL`. |
| `EXAM_PLATFORM_EXAMINER_DASHBOARD_COOKIE_KEY`          | Secret; passed as `COOKIE_KEY`.                       |

### GitHub Actions

| Name                        | Required        | Default | Where Set                  | Description                                              |
| --------------------------- | --------------- | ------- | -------------------------- | -------------------------------------------------------- |
| `DOCR_REGISTRY`             | release         |         | GitHub var                 | DigitalOcean registry name.                              |
| `KOMODO_URL`                | deploy          |         | GitHub var (env `staging`) | `https://<KOMODO_DOMAIN>`.                               |
| `KOMODO_API_KEY`            | release, deploy |         | GitHub secret              | Komodo API key. Needs Execute on the deploy Action only. |
| `KOMODO_API_SECRET`         | release, deploy |         | GitHub secret              | Komodo API secret.                                       |
| `DIGITALOCEAN_USERNAME`     | release         |         | GitHub secret              | DigitalOcean access token email address.                 |
| `DIGITALOCEAN_ACCESS_TOKEN` | release         |         | GitHub secret              | Registry login and `doctl`.                              |
