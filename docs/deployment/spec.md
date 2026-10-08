# Spec

This is the specification describing the deployment of the services. Currently, all services are deployed on a single Digital Ocean Droplet.

## Tooling

- Caddy
- Docker
- GitHub
- Google Cloud
- Komodo

### Caddy

As public services are all hosted on the same VM, Caddy handles routing requests from the various domains to the service ports. The domains are DNS-only, A records registered on Cloudflare, all pointing at the public IP of the VM.

There is a main `deploy/Caddyfile` that is the VM entrypoint interpolating the domains, and reverse-proxying to the LAN ports:

```caddyfile
{$KOMODO_DOMAIN} {
  reverse_proxy core:9120
}

{$EXAMINER_DASHBOARD_DOMAIN} {
  reverse_proxy examiner-dashboard:13003
}
```

All public apps need to be added to this config. The interpolated values come from `deploy/.env`.

TODO: Add actual caddyfile path/ref.

#### Notes

- The Komodo domain proxy should not expose the periphery
  - `404` for `/ws/periphery`

### Docker

All services/components are containerized, and a docker network (`platform`) is created for the services to communicate through. `deploy/bootstrap.sh` creates this network on initial creation.

Komodo manages the container stacks.

### GitHub

The images are built on GHA runners, and pushed to DOCR.

### Google Cloud

The Examiner Dashboard and Exam Platform apps use Google OAuth. These two clients need to be created in Google Cloud Console.

TODO: redirect URIs

#### Notes

- Clients are scoped as being _internal_
- Servers perform check that email matches only `@freecodecamp.org` addresses

### Komodo

Komodo manages the containers. Its own compose file lives in `deploy/`. It does **not** manage itself.
