# Spec

This is the specification describing the deployment of the services. Currently, all services are deployed on a single Digital Ocean Droplet.

TODO: unconsidered: coordinating breaking changes. perhaps a non-issue, provided all changes are backwards compatible - rolling migrations.

## Tooling

- Caddy
- Docker
- GitHub
- Google Cloud
- Komodo
- release-please

### Caddy

As public services are all hosted on the same VM, Caddy handles routing requests from the various domains to the service ports. The domains are DNS-only, A records registered on Cloudflare, all pointing at the public IP of the VM.

The main [`deploy/caddy/Caddyfile`](../../deploy/caddy/Caddyfile) is the VM entrypoint, interpolating domains and reverse-proxying to container aliases and ports:

```caddyfile
{$KOMODO_DOMAIN} {
  respond /ws/periphery 404
  reverse_proxy core:9120
}

{$AUTH_API_DOMAIN} {
  reverse_proxy auth-api:13001
}

{$EXAMINER_DASHBOARD_DOMAIN} {
  reverse_proxy examiner-dashboard:13002
}

{$WEB_APP_DOMAIN} {
  reverse_proxy web-app:13003
}
```

All public apps need to be added to this config. The interpolated values come from `deploy/.env`.

#### Notes

- The Komodo domain proxy should not expose the periphery
  - `404` for `/ws/periphery`

### Docker

All services/components are containerized, and a docker network (`platform`) is created for the services to communicate through. `deploy/bootstrap.sh` creates this network on initial creation.

Komodo manages the container stacks.

### GitHub

The images are built on GHA runners, and pushed to DOCR.

- `docs.yml`
  - on pushes to main where `docs/` changes, GH Pages deployment is made
- `release-container.yml`
  - builds then pushes the containers to DOCR, then calls `deploy.yml`
- `release.yml`
  - release-please pr action
  - calls `release-container.yml`

### Google Cloud

The Examiner Dashboard and Exam Platform apps use Google OAuth. These two clients need to be created in Google Cloud Console.

Dashboard callback: `https://examiner-dashboard.freecodecamp.dev/auth/google/callback`. Optional Komodo callback: `https://exam-platform.freecodecamp.dev/auth/google/callback`. Substitute configured domains; register each URI exactly. See [setup](setup.md#examiner-dashboard-oauth) for required dashboard variables.

#### Notes

- Clients are scoped as being _internal_
- Servers perform check that email matches only `@freecodecamp.org` addresses

### Komodo

Komodo manages the containers. Its own compose file lives in `deploy/`. It does **not** manage itself.

Komodo handles rollbacks by going to the previous healthy image, or just with another GitOps deployment. There is a deploy "Action" in Komodo that is called through the deploy GHA calling a webhook with:

- `APP` - name of app being deployed
- `IMAGE` - registry URL or `previous`

### release-please

There is one release-please config per service. A change to that service creates a release PR. Upon merge, GHA builds the service image and pushes to DOCR.

| Timing                                       | Result                                                                                                                                        |
| -------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------- |
| Run 1 has not started its release-please job | Run 1 releases both. container gets a 2-entry matrix and builds each at its own sha (that app's release PR merge commit). Run 2 finds nothing |
| before merge B                               | pending, refreshes the release PRs, and its matrix is empty.                                                                                  |
| Run 1 already past release-please            | Run 1 releases A only. Run 2 waits, then releases B.                                                                                          |
| Three or more merges                         | Run 2's waiting job is replaced by run 3, which releases whatever is still unreleased. Nothing is lost.                                       |

A release always builds in the same run that created it. So, no release is orphaned and none is built twice. If a release-please step fails, the other apps' releases still build (`continue-on-error`), and the last step fails the job.
