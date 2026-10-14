# Web App (wApp)

The candidate's entry surface, and second device to potentially proctor.

|          |                                           |
| -------- | ----------------------------------------- |
| Dev port | `127.0.0.1:13003`                         |
| Reaches  | aAPI only, at `AUTH_API_URL`              |
| Output   | `dist/`; host must serve `/config.js`     |
| Image    | Caddy serving `dist/` on `13003`          |

```bash
cp .env.example .env
bun --filter web-app dev
bun --filter web-app build
docker compose up --build # reads AUTH_API_URL from .env at runtime
```
