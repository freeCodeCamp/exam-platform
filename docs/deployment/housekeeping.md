# Housekeeping

Komodo is not in the request path. So, a downed Komodo only affects the automated deploy path. Docker restarts the app containers. Caddy is in the path. So, a Caddyfile change reloads without dropping connections - a new Caddy image restarts it.

## Update

- Merge to `main`
- Run `/opt/exam-platform/deploy/bootstrap.sh` on the Droplet.

## Upgrade

### Komodo

- Read the [release notes](https://github.com/moghtech/komodo/releases)
- Bump the `komodo-core` and `komodo-periphery` tags together in `compose.yaml` through a PR
- MongoDB stays on `8.0`: each update pulls its newest patch release
  - A major upgrade waits for Komodo to ship a driver tested against it, then needs MongoDB's `featureCompatibilityVersion` steps

### Caddy

- bump its tag in a PR

### Ubuntu

- it installs security updates by itself
- Reboot when `/var/run/reboot-required` exists; every container restarts with Docker

## Backups

- Droplet backups (`--enable-backups`) image the whole disk weekly.
- Komodo's default **Backup Core Database** Procedure dumps the database daily at 01:00 UTC to `/etc/komodo/backups`, and keeps the newest 14. The dumps are unencrypted and on the Droplet: copy them elsewhere, encrypted, if the Droplet backups are not enough.
- Restore into an empty database: stop Core and Periphery, drop the database, restore, then start them again.

```bash
cd /opt/exam-platform/deploy && set -a && . ./.env && set +a
docker compose stop core periphery
docker compose exec mongo mongosh --quiet -u "$KOMODO_DATABASE_USERNAME" -p "$KOMODO_DATABASE_PASSWORD" \
--authenticationDatabase admin --eval 'db.getSiblingDB("komodo").dropDatabase()'
docker run --rm --network komodo_default -v /etc/komodo/backups:/backups \
-e KOMODO_CLI_DATABASE_TARGET_ADDRESS=mongo:27017 \
-e KOMODO_CLI_DATABASE_TARGET_USERNAME="$KOMODO_DATABASE_USERNAME" \
-e KOMODO_CLI_DATABASE_TARGET_PASSWORD="$KOMODO_DATABASE_PASSWORD" \
-e KOMODO_CLI_DATABASE_TARGET_DB_NAME=komodo \
ghcr.io/moghtech/komodo-cli:2.3.3 km database restore -y
# newest; or --restore-folder <dated folder>
docker compose start core periphery
```

## Disk

- The `exam-platform-prune-images` Procedure removes unused images every Sunday at 03:00 UTC
- Container logs rotate (Docker's `local` driver)
  - Watch `/etc/komodo/backups` and `docker system df`

## Rotate

- CI key: create a new API key for `exam-platform-ci`, replace both GitHub secrets, then delete the old key.
- DOCR token: update the registry account under Settings → Providers.
- App secrets: update the secret Variable, then redeploy the stack. Variables apply at deploy.
- Login passwords: in the UI. `KOMODO_INIT_ADMIN_PASSWORD` only applies to an empty database.
- `KOMODO_JWT_SECRET`: edit `.env`, then `docker compose up -d core`. Everyone is logged out.
- MongoDB password: change it with `mongosh` (`db.changeUserPassword`), then edit `.env` and run `docker compose up -d core`. The init variables only apply to an empty volume.

## Logs

```bash
cd /opt/exam-platform/deploy
docker compose logs -f core periphery caddy
```

Do not:

- Manage this compose project from Komodo.
- Publish MongoDB, or port 9120 beyond `127.0.0.1`.
- Remove the `/ws/periphery` block: no remote servers connect.

## Google OAuth

- Set `KOMODO_GOOGLE_OAUTH_ENABLED=true`, `KOMODO_GOOGLE_OAUTH_ID` and `KOMODO_GOOGLE_OAUTH_SECRET` in `.env`
- `docker compose up -d core`
  - The Komodo UI and API cannot set login providers
- The Google Cloud OAuth client (web application) needs redirect URI `https://<KOMODO_DOMAIN>/auth/google/callback`
- Sign-up stays closed, so users link Google to an account an admin created
