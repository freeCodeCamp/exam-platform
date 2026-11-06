//! Question objects: files (images, audio) uploaded to the curriculum bucket and recorded in cDb's
//! `objects` table on `dev`.
//!
//! Content-addressed: the key is `sha256/<sha256>.<ext>`, so re-uploading the same bytes stores
//! nothing and an object is never overwritten with different content. Listing reads cDb, never the
//! bucket. No delete: every Dolt commit keeps its rows, so an object may be referenced long after
//! `dev` stops pointing at it.
use std::time::Duration;

use axum::{
    Json, Router,
    body::Bytes,
    extract::{DefaultBodyLimit, Multipart, State, multipart::MultipartError},
    http::header::{CACHE_CONTROL, HeaderValue},
    response::IntoResponse,
    routing::get,
};
use cdb::{NewObject, Object, ObjectHash, ObjectKey};
use http::StatusCode;
use object_storage::ObjectStore;
use serde::Serialize;
use time::format_description::well_known::Rfc3339;
use tracing::instrument;

use crate::{config::ServerState, error::Error, extractor::AuthUser};

pub const MAX_FILE_BYTES: usize = 10 * 1024 * 1024;
/// Every part is buffered before any is stored, so this bounds memory per request.
pub const MAX_FILES_PER_REQUEST: usize = 5;
/// Multipart boundaries and part headers.
const MULTIPART_OVERHEAD_BYTES: usize = 64 * 1024;
const MAX_REQUEST_BYTES: usize = MAX_FILES_PER_REQUEST * MAX_FILE_BYTES + MULTIPART_OVERHEAD_BYTES;
/// cDb's `objects.file_name` is `VARCHAR(255)`: characters.
const MAX_FILE_NAME_CHARS: usize = 255;
/// Long enough to render a page of previews, short enough that a leaked link soon dies. The client
/// refetches before it runs out.
const PRESIGNED_URL_TTL: Duration = Duration::from_secs(15 * 60);

pub fn router() -> Router<ServerState> {
    Router::new()
        .route("/", get(get_objects).post(post_objects))
        .layer(DefaultBodyLimit::max(MAX_REQUEST_BYTES))
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StoredObject {
    pub sha256: String,
    /// In the curriculum bucket.
    pub object_key: String,
    pub content_type: String,
    pub byte_size: i32,
    /// As first uploaded.
    pub file_name: String,
    /// RFC 3339.
    pub created_at: String,
    /// Presigned `GET`, valid for [`PRESIGNED_URL_TTL`]. Never store it: store `sha256`.
    pub url: String,
}

impl StoredObject {
    async fn new(object: Object, bucket: &ObjectStore) -> Result<Self, Error> {
        let url = bucket
            .presigned_get(object.object_key.as_str(), PRESIGNED_URL_TTL)
            .await?;
        Ok(Self {
            sha256: object.sha256.into_string(),
            object_key: object.object_key.into_string(),
            content_type: object.content_type,
            byte_size: object.byte_size,
            file_name: object.file_name,
            created_at: object.created_at.assume_utc().format(&Rfc3339)?,
            url,
        })
    }
}

/// Every object on cDb's `dev`, newest first.
#[instrument(skip_all, err(Debug), level = "debug")]
pub async fn get_objects(
    _: AuthUser,
    State(server_state): State<ServerState>,
) -> Result<impl IntoResponse, Error> {
    let objects = server_state.curriculum.list_objects().await?;

    let mut stored = Vec::with_capacity(objects.len());
    for object in objects {
        stored.push(StoredObject::new(object, &server_state.curriculum_bucket).await?);
    }

    Ok((
        // Presigned URLs expire.
        [(CACHE_CONTROL, HeaderValue::from_static("no-store"))],
        Json(stored),
    ))
}

/// A file part that passed every check.
struct Upload {
    sha256: ObjectHash,
    object_key: ObjectKey,
    kind: &'static Kind,
    file_name: String,
    body: Bytes,
}

/// Stores each file part of a `multipart/form-data` body, then records it in cDb. Non-file parts
/// are ignored. Every file is checked before any is stored, so a rejected request stores nothing.
///
/// A file already in cDb is neither re-sent to the bucket nor re-recorded: its response entry is
/// the stored row, with the first upload's file name. Parts are stored in order: when one fails,
/// those before it stay stored, and the error names them. Retrying the request is safe.
#[instrument(skip_all, err(Debug), level = "debug")]
pub async fn post_objects(
    AuthUser(user): AuthUser,
    State(server_state): State<ServerState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, Error> {
    let mut uploads = Vec::new();

    while let Some(mut field) = multipart.next_field().await.map_err(multipart_error)? {
        // A text field: nothing to store.
        let Some(file_name) = field.file_name().map(clean_file_name) else {
            continue;
        };
        if uploads.len() == MAX_FILES_PER_REQUEST {
            return Err(bad_request(format!(
                "at most {MAX_FILES_PER_REQUEST} files per upload"
            )));
        }

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
                format!("{file_name} is not one of: {}", Kind::names()),
            )
        })?;

        let sha256 = ObjectHash::of(&body);
        uploads.push(Upload {
            object_key: ObjectKey::new(&sha256, kind.extension),
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
    let mut stored = Vec::with_capacity(uploads.len());
    for upload in uploads {
        let file_name = upload.file_name.clone();
        match store(&server_state, upload, &author).await {
            Ok(object) => stored.push(object),
            Err(error) => return Err(part_failed(&file_name, error, &stored)),
        }
    }

    Ok((StatusCode::CREATED, Json(stored)))
}

/// Stores `upload` in the bucket and cDb, unless cDb already has it.
async fn store(
    server_state: &ServerState,
    upload: Upload,
    author: &str,
) -> Result<StoredObject, Error> {
    let cdb = &server_state.curriculum;
    let bucket = &server_state.curriculum_bucket;

    if let Some(object) = cdb.get_object(&upload.sha256).await? {
        return StoredObject::new(object, bucket).await;
    }

    let byte_size =
        i32::try_from(upload.body.len()).expect("Unreachable. bounded by MAX_FILE_BYTES");
    // Bucket first: a recorded object always has its bytes. If cDb then fails, the object stays
    // in the bucket unrecorded until a retry records it.
    bucket
        .put(
            upload.object_key.as_str(),
            upload.body,
            upload.kind.content_type,
        )
        .await?;
    // `false`: a concurrent upload recorded it first. Either way, return the stored row.
    cdb.insert_object(
        &NewObject {
            sha256: &upload.sha256,
            object_key: &upload.object_key,
            content_type: upload.kind.content_type,
            byte_size,
            file_name: &upload.file_name,
        },
        author,
    )
    .await?;
    let object = cdb.get_object(&upload.sha256).await?.ok_or_else(|| {
        Error::Server(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("{} recorded but not found", upload.object_key),
        )
    })?;
    StoredObject::new(object, bucket).await
}

/// `error` storing `file_name`, naming what was stored before it.
fn part_failed(file_name: &str, error: Error, stored: &[StoredObject]) -> Error {
    tracing::error!(%error, file_name, "storing an uploaded file failed");
    let message = error.to_string();
    let status = StatusCode::from(error);
    let stored = if stored.is_empty() {
        "nothing".to_string()
    } else {
        stored
            .iter()
            .map(|object| object.file_name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    };
    Error::Server(
        status,
        format!("{file_name}: {message}. Stored before it: {stored}"),
    )
}

fn bad_request(message: String) -> Error {
    Error::Server(StatusCode::BAD_REQUEST, message)
}

fn multipart_error(error: MultipartError) -> Error {
    Error::Server(error.status(), error.body_text())
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
#[derive(Debug, PartialEq, Eq)]
struct Kind {
    /// Shown to examiners.
    name: &'static str,
    content_type: &'static str,
    extension: &'static str,
}

/// Every accepted type. The client's copy is `client/src/utils/objects.ts`.
const KINDS: [Kind; 9] = [
    Kind::new("PNG", "image/png", "png"),
    Kind::new("JPEG", "image/jpeg", "jpg"),
    Kind::new("GIF", "image/gif", "gif"),
    Kind::new("WebP", "image/webp", "webp"),
    Kind::new("MP3", "audio/mpeg", "mp3"),
    Kind::new("WAV", "audio/wav", "wav"),
    Kind::new("Ogg", "audio/ogg", "ogg"),
    Kind::new("FLAC", "audio/flac", "flac"),
    Kind::new("M4A", "audio/mp4", "m4a"),
];

impl Kind {
    const fn new(name: &'static str, content_type: &'static str, extension: &'static str) -> Self {
        Self {
            name,
            content_type,
            extension,
        }
    }

    fn names() -> String {
        KINDS.map(|kind| kind.name).join(", ")
    }

    fn by_extension(extension: &str) -> &'static Self {
        KINDS
            .iter()
            .find(|kind| kind.extension == extension)
            .expect("Unreachable. sniff names only extensions in KINDS")
    }
}

fn sniff(bytes: &[u8]) -> Option<&'static Kind> {
    let at = |offset: usize, magic: &[u8]| bytes.get(offset..offset + magic.len()) == Some(magic);

    let extension = if at(0, b"\x89PNG\r\n\x1a\n") {
        "png"
    } else if at(0, b"\xff\xd8\xff") {
        "jpg"
    } else if at(0, b"GIF87a") || at(0, b"GIF89a") {
        "gif"
    } else if at(0, b"RIFF") && at(8, b"WEBP") {
        "webp"
    } else if at(0, b"RIFF") && at(8, b"WAVE") {
        "wav"
    } else if at(0, b"OggS") {
        "ogg"
    } else if at(0, b"fLaC") {
        "flac"
    } else if at(4, b"ftyp") && (at(8, b"M4A ") || at(8, b"M4B ")) {
        "m4a"
    } else if at(0, b"ID3") || is_mp3_frame_header(bytes) {
        "mp3"
    } else {
        return None;
    };
    Some(Kind::by_extension(extension))
}

/// A valid MPEG audio Layer III frame header: frame sync, then no reserved or free-format field.
/// The sync bits alone also match text, e.g. a UTF-16LE byte order mark (`ff fe`).
fn is_mp3_frame_header(bytes: &[u8]) -> bool {
    let [0xff, b1, b2, ..] = *bytes else {
        return false;
    };
    let sync = b1 & 0xe0 == 0xe0;
    let version = (b1 >> 3) & 0b11; // 0b01: reserved.
    let layer = (b1 >> 1) & 0b11; // 0b01: Layer III.
    let bitrate = b2 >> 4; // 0: free format. 0b1111: invalid.
    let sample_rate = (b2 >> 2) & 0b11; // 0b11: reserved.
    sync && version != 0b01
        && layer == 0b01
        && !matches!(bitrate, 0 | 0b1111)
        && sample_rate != 0b11
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
            // MPEG-1 Layer III, 128 kbit/s, 44.1 kHz.
            (b"\xff\xfb\x90\x00", "mp3"),
            // MPEG-2 Layer III, 64 kbit/s, 22.05 kHz.
            (b"\xff\xf3\x80\x00", "mp3"),
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
            // UTF-16LE text and HTML.
            b"\xff\xfeh\0i\0",
            b"\xff\xfe<\0h\0t\0m\0l\0",
            // Layer I and II.
            b"\xff\xfe\x90\x00",
            b"\xff\xfd\x90\x00",
            // Reserved version, free-format and invalid bitrates, reserved sample rate.
            b"\xff\xeb\x90\x00",
            b"\xff\xfb\x00\x00",
            b"\xff\xfb\xf0\x00",
            b"\xff\xfb\x9c\x00",
        ] {
            assert_eq!(sniff(bytes), None, "{bytes:?}");
        }
    }

    #[test]
    fn every_kind_is_sniffable() {
        for kind in &KINDS {
            assert_eq!(Kind::by_extension(kind.extension), kind);
        }
        assert_eq!(
            Kind::names(),
            "PNG, JPEG, GIF, WebP, MP3, WAV, Ogg, FLAC, M4A"
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
