# Examiner Dashboard

Where examiners author, review, promote, and moderate.

|                 |                                                   |
| --------------- | ------------------------------------------------- |
| `server/`       | Serves the built client from `WEB_DIR`.           |
| `client/`       | Bun workspace member `examiner-dashboard-client`. |
| Server port     | `127.0.0.1:13002`                                 |
| Client dev port | `127.0.0.1:13005`, proxies `/api` to the server   |

## Question assets

`/file-dragger` uploads images and audio for questions through `/api/files`. Each file goes to the curriculum bucket, then gets a row in cDb's `assets` table on `dev`, one Dolt commit per new file, authored by the examiner. `question_assets` links a question to an asset by hash.

- **Content-addressed.** The key is `sha256/<sha256>.<ext>`. The same bytes give the same key, so a re-upload is a no-op and an object is never overwritten with different content.
- **Sniffed.** The type comes from the file's magic bytes: PNG, JPEG, GIF, WebP, MP3, WAV, Ogg, FLAC, M4A. The uploader's content type and file name are ignored. No SVG.
- **Checked before stored.** Up to 5 files of 10 MiB per request. Every part is checked before any is stored, so a rejected request stores nothing. After a failure part way, retrying is safe.
- **Listed from cDb.** eDd never lists the bucket.
- **Private.** The client shows objects through presigned URLs that expire after 15 minutes.
- **Never deleted.** Any cDb commit may reference an object.

The S3 client is `libs/object-storage`, which knows no provider: `CURRICULUM_S3_*` sets the endpoint, region, bucket and key pair. To switch provider, change those values and copy the objects across with their keys.

The Dockerfile builds `client/` with Bun, `server/` with cargo, and copies `dist` to `/app/web`.

`client/src/layouts/base.astro` is the HTML shell. Each `pages/*.astro` is routing only and mounts exactly one `views/*.tsx` React component, which wraps itself in `components/app-root.tsx` (React providers).

```bash
cp .env.example .env
cargo run -p examiner-dashboard
bun --filter examiner-dashboard-client dev
docker compose up --build
```
