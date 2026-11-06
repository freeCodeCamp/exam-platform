# Examiner Dashboard

Where examiners author, review, promote, and moderate.

|                 |                                                   |
| --------------- | ------------------------------------------------- |
| `server/`       | Serves the built client from `WEB_DIR`.           |
| `client/`       | Bun workspace member `examiner-dashboard-client`. |
| Server port     | `127.0.0.1:13002`                                 |
| Client dev port | `127.0.0.1:13005`, proxies `/api` to the server   |

## Question objects

`/file-dragger` uploads images and audio for questions through `/api/objects`. Each file goes to the curriculum bucket, then gets a row in cDb's `objects` table on `dev`, authored by the examiner. That is one Dolt commit per new file, plus a merge commit when `dev` moved during the upload. Nothing else on `dev` is committed with it (see [cDb](./curriculum-database.md#objects)). `question_objects` links a question to an object by hash.

- **Content-addressed.** The key is `sha256/<sha256>.<ext>`. The same bytes give the same key, so an object is never overwritten with different content.
- **Stored once.** A file already in cDb is not re-sent to the bucket or re-recorded. The response returns the stored row, with the first upload's file name.
- **Sniffed.** The type comes from the file's magic bytes: PNG, JPEG, GIF, WebP, MP3, WAV, Ogg, FLAC, M4A. MP3 without an ID3 tag needs a valid Layer III frame header. The uploader's content type and file name are ignored. No SVG. The list lives in `server/routes/objects.rs` (`KINDS`) and `client/src/utils/objects.ts`, which also matches by extension because browsers report some types under other names.
- **Checked before stored.** Up to 5 files of 10 MiB per request; non-file form fields are ignored. Every file is checked before any is stored, so a rejected request stores nothing.
- **Stored in order.** When a file fails, those before it stay stored and the error names them. Retrying is safe. If cDb fails after the bucket write, the object stays in the bucket unrecorded and unlisted until a retry records it.
- **Listed from cDb.** eDd never lists the bucket.
- **Private.** The client shows objects through presigned URLs that expire after 15 minutes, and refetches the list every 10.
- **Never deleted.** Any cDb commit may reference an object.

The S3 client is `libs/object-storage`, which knows no provider: `CURRICULUM_S3_*` sets the endpoint, region, bucket and key pair. To switch provider, change those values and copy the objects across with their keys. Local development: `bun run dev edd` points eDd at SeaweedFS (`compose.dev.yaml`, [Run](../CONTRIBUTING.md#run)).

The Dockerfile builds `client/` with Bun, `server/` with cargo, and copies `dist` to `/app/web`.

`client/src/layouts/base.astro` is the HTML shell. Each `pages/*.astro` is routing only and mounts exactly one `views/*.tsx` React component, which wraps itself in `components/app-root.tsx` (React providers).

```bash
cp .env.example .env
cargo run -p examiner-dashboard
bun --filter examiner-dashboard-client dev
docker compose up --build
```
