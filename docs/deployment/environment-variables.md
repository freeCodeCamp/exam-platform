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

### `apps/auth-api`

- `ENVIRONMENT`
  - required
- `SENTRY_DSN`
  - required outside development
- `SENTRY_TRACES_SAMPLE_RATE`
- `RUST_LOG`
- `AAPI_PORT`
  - `13001`
- `PLATFORM_NETWORK`
  - `platform`
- `IMAGE`
  - leave empty to build from source. Komodo sets on deploys.

### `apps/examiner-dashboard`

- `ENVIRONMENT`
  - required
- `SENTRY_DSN`
  - required outside development
- `SENTRY_TRACES_SAMPLE_RATE`
- `RUST_LOG`
- `EDD_PORT`
  - `13003`
- `PLATFORM_NETWORK`
  - `platform`
- `IMAGE`
  - leave empty to build from source. Komodo sets on deploys.

### `apps/web-app`

- `PUBLIC_AUTH_API_URL`
  - build time
  - auth API origin the browser calls
  - `http://127.0.0.1:13001`
- `WAPP_PORT`
  - `13004`
- `PLATFORM_NETWORK`
  - `platform`
- `IMAGE`

### `apps/desktop-app`

- `ENVIRONMENT`
  - compile time
  - `production`
- `SENTRY_DSN`
  - compile time
  - required unless `ENVIRONMENT=development`
- `AUTH_API_URL`
  - required unless `ENVIRONMENT=development`
  - `http://127.0.0.1:13001`

### `deploy/`

- `KOMODO_DOMAIN`
- `EXAMINER_DASHBOARD_DOMAIN`
- `KOMODO_INIT_ADMIN_USERNAME`
- `KOMODO_INIT_ADMIN_PASSWORD`
- `KOMODO_DATABASE_USERNAME`
- `KOMODO_DATABASE_PASSWORD`
- `KOMODO_JWT_SECRET`
- `KOMODO_WEBHOOK_SECRET`
- `KOMODO_GOOGLE_OAUTH_ENABLED`
- `KOMODO_GOOGLE_OAUTH_ID`
- `KOMODO_GOOGLE_OAUTH_SECRET`

- `bootstrap.sh` auto-generates the empty secrets with openssl rand -hex 32.
