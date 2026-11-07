# Examiner Dashboard

Where examiners author, review, promote, and moderate.

|                 |                                                   |
| --------------- | ------------------------------------------------- |
| `server/`       | Serves the built client from `WEB_DIR`.           |
| `client/`       | Bun workspace member `examiner-dashboard-client`. |
| Server port     | `127.0.0.1:13002`                                 |
| Client dev port | `127.0.0.1:13005`, proxies `/api` to the server   |

The S3 client is `libs/object-storage`, which knows no provider: `CURRICULUM_S3_*` sets the endpoint, region, bucket and key pair. To switch provider, change those values and copy the objects across with their keys. Local development: `bun run dev edd` points eDd at SeaweedFS (`compose.dev.yaml`, [Run](../CONTRIBUTING.md#run)).

The Dockerfile builds `client/` with Bun, `server/` with cargo, and copies `dist` to `/app/web`.

`client/src/layouts/base.astro` is the HTML shell. Each `pages/*.astro` is routing only and mounts exactly one `views/*.tsx` React component, which wraps itself in `components/app-root.tsx` (React providers).

```bash
cp .env.example .env
cargo run -p examiner-dashboard
bun --filter examiner-dashboard-client dev
docker compose up --build
```

## Question objects

Examiners upload question images and audio on the file dragger page (`/api/objects`). Each file is content-addressed in the curriculum bucket at `sha256/<sha256>.<ext>` and recorded in eDb's `edd.objects`. Re-uploading the same bytes stores nothing.

The object store page (`/api/object-store`) lists the bucket joined with `edd.objects`, so strays on either side show. Examiners rename recorded objects and delete any key. Rename and delete take the key in the `key` query parameter, never the path, so the browser cannot normalize a key such as `a/../b` into another one.

Deleting is permanent and unchecked: a cDb commit that references the object loses its bytes. The page makes the examiner confirm first.

Upload and delete of a key each hold a Postgres advisory lock on that key (`edb::ObjectKeyLock`) across their bucket and eDb writes, so they serialize across eDd instances.
