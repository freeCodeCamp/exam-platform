//! Question objects: files (images, audio) uploaded to the curriculum bucket and recorded in eDb's
//! `edd.objects`. An upload is not curriculum content: its row reaches cDb only with the content
//! commit that first references it.
//!
//! Content-addressed: the key is `sha256/<sha256>.<ext>`, so re-uploading the same bytes stores
//! nothing and an object is never overwritten with different content. Listing reads eDb, never the
//! bucket. Renaming and deleting are the object store's (`object_store.rs`).
use axum::{
    Json, Router,
    body::Bytes,
    extract::{DefaultBodyLimit, Multipart, State, multipart::MultipartError},
    response::IntoResponse,
    routing::get,
};
use edb::{NewObject, Object, ObjectHash, ObjectKey};
use http::StatusCode;
use object_storage::ObjectStore;
use serde::Serialize;
use sqlx::types::Uuid;
use tracing::instrument;

use crate::{
    config::ServerState,
    error::Error,
    extractor::AuthUser,
    objects::{ObjectRecord, clean_file_name, presigned_get, uncached_json},
};

pub const MAX_FILE_BYTES: usize = 10 * 1024 * 1024;
/// Every part is buffered before any is stored, so this bounds memory per request.
pub const MAX_FILES_PER_REQUEST: usize = 5;
/// Multipart boundaries and part headers.
const MULTIPART_OVERHEAD_BYTES: usize = 64 * 1024;
const MAX_REQUEST_BYTES: usize = MAX_FILES_PER_REQUEST * MAX_FILE_BYTES + MULTIPART_OVERHEAD_BYTES;

pub fn router() -> Router<ServerState> {
    Router::new()
        .route("/", get(get_objects).post(post_objects))
        .layer(DefaultBodyLimit::max(MAX_REQUEST_BYTES))
}

/// A recorded object: its eDb record, with its bytes' key and a link to them.
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RecordedObject {
    #[serde(flatten)]
    pub record: ObjectRecord,
    /// In the curriculum bucket.
    pub object_key: String,
    pub byte_size: i32,
    /// See [`presigned_get`]. Never store it: store `sha256`.
    pub url: String,
}

impl RecordedObject {
    async fn new(object: Object, bucket: &ObjectStore) -> Result<Self, Error> {
        Ok(Self {
            record: ObjectRecord::new(&object)?,
            url: presigned_get(bucket, object.object_key.as_str()).await?,
            object_key: object.object_key.into_string(),
            byte_size: object.byte_size,
        })
    }
}

/// Every uploaded object, newest first.
#[instrument(skip_all, err(Debug), level = "debug")]
pub async fn get_objects(
    _: AuthUser,
    State(server_state): State<ServerState>,
) -> Result<impl IntoResponse, Error> {
    let objects = server_state.database.list_objects().await?;

    let mut recorded = Vec::with_capacity(objects.len());
    for object in objects {
        recorded.push(RecordedObject::new(object, &server_state.curriculum_bucket).await?);
    }

    Ok(uncached_json(recorded))
}

/// A file part that passed every check.
struct Upload {
    sha256: ObjectHash,
    object_key: ObjectKey,
    kind: &'static Kind,
    file_name: String,
    body: Bytes,
}

/// Stores each file part of a `multipart/form-data` body, then records it in eDb. Non-file parts
/// are ignored. Every file is checked before any is stored, so a rejected request stores nothing.
///
/// A file already in eDb is neither re-sent to the bucket nor re-recorded: its response entry is
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
            return Err(Error::bad_request(format!(
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
            return Err(Error::bad_request(format!("{file_name} is empty")));
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
        return Err(Error::bad_request("no files in upload"));
    }

    let mut stored = Vec::with_capacity(uploads.len());
    for upload in uploads {
        let file_name = upload.file_name.clone();
        match store(&server_state, upload, user.id).await {
            Ok(object) => stored.push(object),
            Err(error) => return Err(part_failed(&file_name, error, &stored)),
        }
    }

    Ok((StatusCode::CREATED, Json(stored)))
}

/// Stores `upload` in the bucket and eDb, unless eDb already has it. Holds the key's lock
/// throughout, so a concurrent delete cannot remove the bytes between the check and the record.
async fn store(
    server_state: &ServerState,
    upload: Upload,
    uploaded_by: Uuid,
) -> Result<RecordedObject, Error> {
    let bucket = &server_state.curriculum_bucket;
    let mut lock = server_state
        .database
        .lock_object_key(upload.object_key.as_str())
        .await?;

    if let Some(object) = lock.get_object(&upload.sha256).await? {
        lock.commit().await?;
        return RecordedObject::new(object, bucket).await;
    }

    let byte_size =
        i32::try_from(upload.body.len()).expect("Unreachable. bounded by MAX_FILE_BYTES");
    // Bucket first: a recorded object always has its bytes. If eDb then fails, the object stays
    // in the bucket unrecorded until a retry records it.
    bucket
        .put(
            upload.object_key.as_str(),
            upload.body,
            upload.kind.content_type,
        )
        .await?;
    lock.insert_object(&NewObject {
        sha256: &upload.sha256,
        object_key: &upload.object_key,
        content_type: upload.kind.content_type,
        byte_size,
        file_name: &upload.file_name,
        uploaded_by,
    })
    .await?;
    let object = lock.get_object(&upload.sha256).await?.ok_or_else(|| {
        Error::Server(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("{} recorded but not found", upload.object_key),
        )
    })?;
    lock.commit().await?;
    RecordedObject::new(object, bucket).await
}

/// `error` storing `file_name`, naming what was stored before it.
fn part_failed(file_name: &str, error: Error, stored: &[RecordedObject]) -> Error {
    tracing::error!(%error, file_name, "storing an uploaded file failed");
    let message = error.to_string();
    let status = StatusCode::from(error);
    let stored = if stored.is_empty() {
        "nothing".to_string()
    } else {
        stored
            .iter()
            .map(|object| object.record.file_name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    };
    Error::Server(
        status,
        format!("{file_name}: {message}. Stored before it: {stored}"),
    )
}

fn multipart_error(error: MultipartError) -> Error {
    Error::Server(error.status(), error.body_text())
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
}
