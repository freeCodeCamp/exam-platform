# Examiner Dashboard (eDd)

Where examiners author, review, promote, and moderate.

|                 |                                                                                                                                                                                          |
| --------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `server/`       | Rust, Axum, Tokio. Crate `examiner-dashboard` (root Cargo workspace). Serves the built client from `WEB_DIR`.                                                                            |
| `client/`       | TypeScript, Astro (static), React, Chakra UI, TanStack Query. Config (`package.json`, `astro.config.ts`, `tsconfig.json`) in app root; Bun workspace member `examiner-dashboard-client`. |
| Server port     | `127.0.0.1:13002`                                                                                                                                                                        |
| Client dev port | `127.0.0.1:13005`, proxies `/api` to the server                                                                                                                                          |

One image holds both: the Dockerfile builds `client/` with Bun, `server/` with cargo, and copies `dist` to `/app/web`.

Client layout (`client/src/`): `layouts/base.astro` is the HTML shell; each `pages/*.astro` is routing only and mounts exactly one `views/*.tsx` React component, which wraps itself in `components/app-root.tsx` (Chakra + TanStack Query providers). Chakra snippets live in `components/ui/`.

```bash
cp .env.example .env
cargo run -p examiner-dashboard
bun --filter examiner-dashboard-client dev
docker compose up --build
```
