# Auth API

Candidate identity and the attempt path. The only application candidate devices reach.

|        |                                   |
| ------ | --------------------------------- |
| Port   | `127.0.0.1:13001`                 |
| Crate  | `auth-api` (root Cargo workspace) |
| Health | `GET /healthz`                    |

```bash
# Use solution like direnv to have .env read to env
cp .env.example .env
cargo run -p auth-api
docker compose up --build
```
