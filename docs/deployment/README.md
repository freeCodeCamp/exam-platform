# Deployment

Follow this to deploy the platform's services.

## Once-off

Create the VM. A 2vCPU, 4GB RAM, 40GB disk instance is enough to get started.

Create a firewall rule to accept: Incoming SSH:22, HTTP:80, HTTPS:443; Outgoing ICMP, TCP:ALL, UDP:ALL

Ideally, create the VM in the same region as the image registry.

Create A records for all public domains pointing as DNS-only to the VM IPv4.

### SSH and Bootstrap

```bash
ssh root@<vm-ip>
git clone https://github.com/freeCodeCamp/exam-platform.git /opt/exam-platform
/opt/exam-platform/deploy/bootstrap.sh
```

NOTE: `bootstrap.sh` needs to be re-run any time there is a change to the Caddyfile.

### Komodo Log-In

1. Open `https://exam-platform.freecodecamp.dev` and log in as `admin` with the password from `.env`. Change the password, then add a passkey or TOTP. Nobody can sign up: admins add users under Settings → Users, and each user adds 2FA at first login.
2. Check that server `exam-platform` reports OK. Its Periphery connects on start.
3. Settings → Providers: add an image registry account for `registry.digitalocean.com` with a DigitalOcean token scoped to registry read.
4. Settings → Variables: create secret Variables `EXAM_PLATFORM_AUTH_API_SENTRY_DSN`, `EXAM_PLATFORM_EXAMINER_DASHBOARD_SENTRY_DSN`.
5. Settings → Users: create service user `exam-platform-ci` and an API key for it. Store the key in GitHub: [Setup](../docs/releases.md#setup).
6. Syncs → new Resource Sync:
   - Repository `freeCodeCamp/exam-platform`, branch `main`, resource path `deploy/exam-platform.toml`.
   - Include user groups on, include variables off, delete off.
   - Execute it twice. The first run creates the Action after the user group, so the group's Execute permission on it only applies on the second; until then, CI's `RunAction` fails with "User does not have required permissions on this Action". Later edits to the file apply when an admin executes the sync again; it lists pending changes.

The first deploy of each app creates its `EXAM_PLATFORM_<APP>_IMAGE` Variables. Until eDd's first deploy, its name answers 502.
