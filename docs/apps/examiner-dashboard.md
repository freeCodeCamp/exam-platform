# Examiner Dashboard

Where examiners author, review, promote, and moderate.

|                 |                                                   |
| --------------- | ------------------------------------------------- |
| `server/`       | Serves the built client from `WEB_DIR`.           |
| `client/`       | Bun workspace member `examiner-dashboard-client`. |
| Server port     | `127.0.0.1:13002`                                 |
| Client dev port | `127.0.0.1:13005`, proxies `/api` to the server   |

The Dockerfile builds `client/` with Bun, `server/` with cargo, and copies `dist` to `/app/web`.

`client/src/layouts/base.astro` is the HTML shell. Each `pages/*.astro` is routing only and mounts exactly one `views/*.tsx` React component, which wraps itself in `components/app-root.tsx` (React providers).

```bash
cp .env.example .env
cargo run -p examiner-dashboard
bun --filter examiner-dashboard-client dev
docker compose up --build
```
