# Setup

## GitHub

- `Settings → Actions → General`: allow GitHub Actions to create and approve pull requests for release-please.
- Environment `production`: required reviewers, deployment branch `main`.
  - Secrets `KOMODO_API_KEY` and `KOMODO_API_SECRET`: the API key of Komodo service user `exam-platform-ci`.
  - Variable `KOMODO_URL`: `https://exam-platform.freecodecamp.dev`.
- Repository secret `DIGITALOCEAN_ACCESS_TOKEN`: DigitalOcean token with registry read/write.
- Repository variables:
  - `DOCR_REGISTRY`: the registry name.
  - `PUBLIC_AUTH_API_URL`: the aAPI origin wApp's image bakes in.
- Branch protection: require `CI passed`. Release PRs opened with `GITHUB_TOKEN` trigger no CI, so either allow merging them without the check, or give release-please a GitHub App token.

The secrets and variables, with the GitHub CLI:

```bash
gh secret set KOMODO_API_KEY --env production
gh secret set KOMODO_API_SECRET --env production
gh variable set KOMODO_URL --env production --body https://exam-platform.freecodecamp.dev
gh secret set DIGITALOCEAN_ACCESS_TOKEN
gh variable set DOCR_REGISTRY --body <registry>
gh variable set PUBLIC_AUTH_API_URL --body <aAPI origin>
```
