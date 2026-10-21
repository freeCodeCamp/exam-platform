# Setup

## DigitalOcean

Create the VM. A 2vCPU, 4GB RAM, 40GB disk instance is enough to get started.

Create a firewall rule to accept: Incoming SSH:22, HTTP:80, HTTPS:443; Outgoing ICMP, TCP:ALL, UDP:ALL

Ideally, create the VM in the same region as the image registry.

## Cloudflare

Create A records for all public domains pointing as DNS-only to the VM IPv4.

## SSH and Bootstrap

```bash
ssh root@<vm-ip>
git clone https://github.com/freeCodeCamp/exam-platform.git /opt/exam-platform
/opt/exam-platform/deploy/bootstrap.sh
```

NOTE: `bootstrap.sh` needs to be re-run any time there is a change to the Caddyfile.

## Google Login (Optional)

Komodo reads login providers from Core's environment only - the Komodo UI and API cannot set them. `bootstrap.sh` leaves them empty and disabled.

1. Google Cloud Console: create an OAuth client (web application) with redirect URI `https://exam-platform.freecodecamp.dev/auth/google/callback`.
2. On the Droplet, set in `/opt/exam-platform/deploy/.env`:

```bash
KOMODO_GOOGLE_OAUTH_ENABLED=true
KOMODO_GOOGLE_OAUTH_ID=<client_id>
KOMODO_GOOGLE_OAUTH_SECRET=<client_secret>
```

3. Restart Core

```bash
cd /opt/exam-platform/deploy
docker compose up -d core
```

NOTE: Re-running `bootstrap.sh` also applies it.

Admin still needs to create users, because sign-up is closed. Once a user logs in with password, they can link their Google account.

## Examiner Dashboard OAuth

Create a separate Google OAuth web client and register `https://examiner-dashboard.freecodecamp.dev/auth/google/callback`. Login checks for a verified `@freecodecamp.org` Google Workspace account.

Generate cookie key with `openssl rand -hex 64`. Create these Komodo Variables before syncing/deploying the dashboard:

| Variable                                               | Value                            |
| ------------------------------------------------------ | -------------------------------- |
| `EXAM_PLATFORM_EXAMINER_DASHBOARD_GOOGLE_OAUTH_ID`     | OAuth client ID                  |
| `EXAM_PLATFORM_EXAMINER_DASHBOARD_GOOGLE_OAUTH_SECRET` | OAuth client secret; mark secret |
| `EXAM_PLATFORM_EXAMINER_DASHBOARD_GOOGLE_REDIRECT_URL` | Registered callback URL above    |
| `EXAM_PLATFORM_EXAMINER_DASHBOARD_COOKIE_KEY`          | Generated key; mark secret       |

## Examiner Database

Until production moves to hosted Postgres, eDb runs as Komodo stack `exam-platform-examiner-db` (`databases/examiner/compose.yaml`). Generate a password with `openssl rand -hex 32`, then create secret Variables:

- `EXAM_PLATFORM_EXAMINER_DB_PASSWORD`: the password.
- `EXAM_PLATFORM_EXAMINER_DASHBOARD_EDB_DATABASE_URL`: `postgres://edd:<password>@examiner-db:5432/examiner`.

Deploy the eDb stack by hand before eDd's first deploy. The deploy Action does not touch it. eDd migrates the schema on startup.

## Komodo Log-In

1. Open `https://exam-platform.freecodecamp.dev` and log in as `admin` with the password from `.env`. Change the password, then add a passkey or TOTP. Nobody can sign up: admins add users under Settings → Users, and each user adds 2FA at first login.
2. Check that server `exam-platform` reports OK. Its Periphery connects on start.
3. Settings → Providers: add an image registry account named `exam-platform` for `registry.digitalocean.com` with a DigitalOcean token scoped to registry read.
4. Settings → Variables: create secret Variables `EXAM_PLATFORM_AUTH_API_SENTRY_DSN`, `EXAM_PLATFORM_EXAMINER_DASHBOARD_SENTRY_DSN`.
5. Settings → Users: create service user `exam-platform-ci` and an API key for it. Store the key in GitHub: [Setup](../releases.md).
6. Syncs → new Resource Sync:
   - Repository `freeCodeCamp/exam-platform`, branch `main`, resource path `deploy/exam-platform.toml`.
   - Include user groups on, include variables off, delete off.
   - Execute it twice. The first run creates the Action after the user group, so the group's Execute permission on it only applies on the second; until then, CI's `RunAction` fails with "User does not have required permissions on this Action". Later edits to the file apply when an admin executes the sync again; it lists pending changes.

The first deploy of each app creates its `EXAM_PLATFORM_<APP>_IMAGE` Variables. Until eDd's first deploy, its name answers 502.

## GitHub

- `Settings → Actions → General`: allow GitHub Actions to create and approve pull requests for release-please.
- Environment `staging`: required reviewers, deployment branch `main`.
  - Secrets `KOMODO_API_KEY` and `KOMODO_API_SECRET`: the API key of Komodo service user `exam-platform-ci`.
  - Variable `KOMODO_URL`: `https://exam-platform.freecodecamp.dev`.
- Repository secret `DIGITALOCEAN_USERNAME`: DigitalOcean account email used for registry login.
- Repository secret `DIGITALOCEAN_ACCESS_TOKEN`: DigitalOcean token with registry read/write.
- Repository variable `DOCR_REGISTRY`: the registry name.
- Branch protection: require `CI passed`. Release PRs opened with `GITHUB_TOKEN` trigger no CI, so either allow merging them without the check, or give release-please a GitHub App token.

The secrets and variables, with the GitHub CLI:

```bash
gh secret set KOMODO_API_KEY --env staging
gh secret set KOMODO_API_SECRET --env staging
gh variable set KOMODO_URL --env staging --body https://exam-platform.freecodecamp.dev
gh secret set DIGITALOCEAN_USERNAME
gh secret set DIGITALOCEAN_ACCESS_TOKEN
gh variable set DOCR_REGISTRY --body <registry>
```
