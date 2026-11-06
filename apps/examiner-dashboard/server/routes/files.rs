//! Question assets: files (images, audio) uploaded to the curriculum bucket and recorded in cDb's
//! `assets` table on `dev`.
//!
//! Content-addressed: the key is `sha256/<sha256>.<ext>`, so re-uploading the same bytes is a no-op
//! and an object is never overwritten with different content. Listing reads cDb, never the bucket.
//! No delete: every Dolt commit keeps its rows, so an object may be referenced long after `dev`
//! stops pointing at it.
use std::time::Duration;

use axum::{
    Json, Router,
    body::Bytes,
    extract::{DefaultBodyLimit, Multipart, State, multipart::MultipartError},
    http::header::{CACHE_CONTROL, HeaderValue},
    response::IntoResponse,
    routing::get,
};
use cdb::{Asset, NewAsset};
use http::StatusCode;
use serde::Serialize;
use sha2::{Digest, Sha256};
use time::format_description::well_known::Rfc3339;
use tracing::instrument;

use crate::{config::ServerState, error::Error, extractor::AuthUser};

pub const MAX_FILE_BYTES: usize = 10 * 1024 * 1024;
/// Every part is buffered before any is stored, so this bounds memory per request.
pub const MAX_FILES_PER_REQUEST: usize = 5;
/// Multipart boundaries and part headers.
const MULTIPART_OVERHEAD_BYTES: usize = 64 * 1024;
const MAX_REQUEST_BYTES: usize = MAX_FILES_PER_REQUEST * MAX_FILE_BYTES + MULTIPART_OVERHEAD_BYTES;
/// cDb's `assets.file_name` is `VARCHAR(255)`: characters.
const MAX_FILE_NAME_CHARS: usize = 255;
/// Long enough to render a page of previews, short enough that a leaked link soon dies.
const PRESIGNED_URL_TTL: Duration = Duration::from_secs(15 * 60);

pub fn router() -> Router<ServerState> {
    Router::new()
        .route("/", get(get_files).post(post_files))
        .layer(DefaultBodyLimit::max(MAX_REQUEST_BYTES))
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StoredFile {
    pub sha256: String,
    /// Object key in the curriculum bucket.
    pub key: String,
    pub content_type: String,
    pub size: i32,
    pub file_name: String,
    /// RFC 3339. `None` in an upload's response.
    pub created_at: Option<String>,
    /// Presigned `GET`, valid for [`PRESIGNED_URL_TTL`]. Never store it: store `sha256`.
    pub url: String,
}

/// Every asset on cDb's `dev`, newest first.
#[instrument(skip_all, err(Debug), level = "debug")]
pub async fn get_files(
    _: AuthUser,
    State(server_state): State<ServerState>,
) -> Result<impl IntoResponse, Error> {
    let assets = server_state.curriculum.list_assets().await?;

    let mut files = Vec::with_capacity(assets.len());
    for Asset {
        sha256,
        object_key,
        content_type,
        byte_size,
        file_name,
        created_at,
    } in assets
    {
        let url = server_state
            .curriculum_store
            .presigned_get(&object_key, PRESIGNED_URL_TTL)
            .await?;
        files.push(StoredFile {
            sha256,
            key: object_key,
            content_type,
            size: byte_size,
            file_name,
            created_at: created_at.assume_utc().format(&Rfc3339).ok(),
            url,
        });
    }

    Ok((
        // Presigned URLs expire.
        [(CACHE_CONTROL, HeaderValue::from_static("no-store"))],
        Json(files),
    ))
}

/// A part that passed every check.
struct Upload {
    sha256: String,
    key: String,
    kind: Kind,
    file_name: String,
    body: Bytes,
}

/// Stores each part of a `multipart/form-data` body, then records it in cDb. Every part is checked
/// before any is stored, so a rejected request stores nothing. Storing is idempotent: after a
/// failure part way, retrying the request is safe.
#[instrument(skip_all, err(Debug), level = "debug")]
pub async fn post_files(
    AuthUser(user): AuthUser,
    State(server_state): State<ServerState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, Error> {
    let mut uploads = Vec::new();

    while let Some(mut field) = multipart.next_field().await.map_err(multipart_error)? {
        if uploads.len() == MAX_FILES_PER_REQUEST {
            return Err(bad_request(format!(
                "at most {MAX_FILES_PER_REQUEST} files per upload"
            )));
        }

        let file_name = clean_file_name(field.file_name().unwrap_or_default());
        let mut body = Vec::new();
        while let Some(chunk) = field.chunk().await.map_err(multipart_error)? {
            if body.len() + chunk.len() > MAX_FILE_BYTES {
                return Err(Error::Server(
                    StatusCode::PAYLOAD_TOO_LARGE,
                    format!("{file_name} is over {} MiB", MAX_FILE_BYTES / (1024 * 1024)),
                ));
            }
            body.extend_from_slice(&chunk);
        }
        if body.is_empty() {
            return Err(bad_request(format!("{file_name} is empty")));
        }
        let kind = sniff(&body).ok_or_else(|| {
            Error::Server(
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                format!("{file_name} is not one of: {}", Kind::NAMES),
            )
        })?;

        let sha256 = hex_sha256(&body);
        uploads.push(Upload {
            key: format!("sha256/{sha256}.{}", kind.extension),
            sha256,
            kind,
            file_name,
            body: Bytes::from(body),
        });
    }

    if uploads.is_empty() {
        return Err(bad_request("no files in upload".to_string()));
    }

    let author = author(&user.name, &user.email);
    let mut files = Vec::with_capacity(uploads.len());
    for upload in uploads {
        let size =
            i32::try_from(upload.body.len()).expect("Unreachable. bounded by MAX_FILE_BYTES");
        let store = &server_state.curriculum_store;
        // Object first: a recorded asset always has its bytes.
        store
            .put(&upload.key, upload.body, upload.kind.content_type)
            .await?;
        server_state
            .curriculum
            .insert_asset(
                &NewAsset {
                    sha256: &upload.sha256,
                    object_key: &upload.key,
                    content_type: upload.kind.content_type,
                    byte_size: size,
                    file_name: &upload.file_name,
                },
                &author,
            )
            .await?;
        let url = store.presigned_get(&upload.key, PRESIGNED_URL_TTL).await?;

        files.push(StoredFile {
            sha256: upload.sha256,
            key: upload.key,
            content_type: upload.kind.content_type.to_owned(),
            size,
            file_name: upload.file_name,
            created_at: None,
            url,
        });
    }

    Ok((StatusCode::CREATED, Json(files)))
}

fn bad_request(message: String) -> Error {
    Error::Server(StatusCode::BAD_REQUEST, message)
}

fn multipart_error(error: MultipartError) -> Error {
    Error::Server(error.status(), error.body_text())
}

fn hex_sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Shown to examiners and kept in cDb: no control characters, at most [`MAX_FILE_NAME_CHARS`].
fn clean_file_name(raw: &str) -> String {
    let name: String = raw
        .chars()
        .filter(|c| !c.is_control())
        .take(MAX_FILE_NAME_CHARS)
        .collect();
    let name = name.trim();
    if name.is_empty() {
        "unnamed".to_string()
    } else {
        name.to_string()
    }
}

/// Dolt's `Name <email>`. The name loses anything that would end it early.
fn author(name: &str, email: &str) -> String {
    let name: String = name
        .chars()
        .filter(|c| !c.is_control() && !matches!(c, '<' | '>'))
        .collect();
    let name = name.trim();
    let name = if name.is_empty() { email } else { name };
    format!("{name} <{email}>")
}

/// An accepted file type. Sniffed from the bytes: the uploader's content type and file name are
/// never trusted. No SVG: it can carry script.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Kind {
    content_type: &'static str,
    extension: &'static str,
}

impl Kind {
    const NAMES: &str = "PNG, JPEG, GIF, WebP, MP3, WAV, Ogg, FLAC, M4A";

    const fn new(content_type: &'static str, extension: &'static str) -> Self {
        Self {
            content_type,
            extension,
        }
    }
}

fn sniff(bytes: &[u8]) -> Option<Kind> {
    let at = |offset: usize, magic: &[u8]| bytes.get(offset..offset + magic.len()) == Some(magic);

    if at(0, b"\x89PNG\r\n\x1a\n") {
        Some(Kind::new("image/png", "png"))
    } else if at(0, b"\xff\xd8\xff") {
        Some(Kind::new("image/jpeg", "jpg"))
    } else if at(0, b"GIF87a") || at(0, b"GIF89a") {
        Some(Kind::new("image/gif", "gif"))
    } else if at(0, b"RIFF") && at(8, b"WEBP") {
        Some(Kind::new("image/webp", "webp"))
    } else if at(0, b"RIFF") && at(8, b"WAVE") {
        Some(Kind::new("audio/wav", "wav"))
    } else if at(0, b"OggS") {
        Some(Kind::new("audio/ogg", "ogg"))
    } else if at(0, b"fLaC") {
        Some(Kind::new("audio/flac", "flac"))
    } else if at(4, b"ftyp") && (at(8, b"M4A ") || at(8, b"M4B ")) {
        Some(Kind::new("audio/mp4", "m4a"))
    } else if at(0, b"ID3") || matches!(bytes, [0xff, b, ..] if b & 0xe0 == 0xe0) {
        // ID3 tag, or an MPEG audio frame sync. After JPEG: `ff d8` is not a frame sync.
        Some(Kind::new("audio/mpeg", "mp3"))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sniffs_accepted_types() {
        let cases: &[(&[u8], &str)] = &[
            (b"\x89PNG\r\n\x1a\n....", "png"),
            (b"\xff\xd8\xff\xe0....", "jpg"),
            (b"GIF89a....", "gif"),
            (b"RIFF\0\0\0\0WEBPVP8 ", "webp"),
            (b"RIFF\0\0\0\0WAVEfmt ", "wav"),
            (b"OggS\0\x02", "ogg"),
            (b"fLaC\0\0", "flac"),
            (b"\0\0\0\x20ftypM4A \0\0", "m4a"),
            (b"ID3\x04\0", "mp3"),
            (b"\xff\xfb\x90\x00", "mp3"),
        ];
        for (bytes, extension) in cases {
            assert_eq!(
                sniff(bytes).map(|k| k.extension),
                Some(*extension),
                "{bytes:?}"
            );
        }
    }

    #[test]
    fn rejects_other_types() {
        for bytes in [
            &b"<svg xmlns=\"http://www.w3.org/2000/svg\"/>"[..],
            b"<!doctype html><script>",
            b"%PDF-1.7",
            b"\0\0\0\x20ftypisom",
            b"RIFF\0\0\0\0AVI ",
            b"\xff",
            b"",
        ] {
            assert_eq!(sniff(bytes), None, "{bytes:?}");
        }
    }

    #[test]
    fn hashes_as_lowercase_hex() {
        assert_eq!(
            hex_sha256(b"hello"),
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn cleans_file_names() {
        assert_eq!(clean_file_name(" diagram\n.png "), "diagram.png");
        assert_eq!(clean_file_name(""), "unnamed");
        assert_eq!(clean_file_name("\u{7}"), "unnamed");
        assert_eq!(clean_file_name(&"é".repeat(300)).chars().count(), 255);
    }

    #[test]
    fn authors_stay_well_formed() {
        assert_eq!(
            author("Ada Lovelace", "ada@freecodecamp.org"),
            "Ada Lovelace <ada@freecodecamp.org>"
        );
        assert_eq!(
            author("Eve <x@y>\n", "eve@freecodecamp.org"),
            "Eve x@y <eve@freecodecamp.org>"
        );
        assert_eq!(
            author("", "a@freecodecamp.org"),
            "a@freecodecamp.org <a@freecodecamp.org>"
        );
    }
}
