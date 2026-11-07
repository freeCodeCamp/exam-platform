//! What the question object routes share: [`crate::routes::objects`] uploads and lists recorded
//! objects, [`crate::routes::object_store`] lists the whole bucket and renames and deletes.
use std::time::Duration;

use axum::{
    Json,
    http::header::{CACHE_CONTROL, HeaderValue},
    response::IntoResponse,
};
use edb::Object;
use object_storage::ObjectStore;
use serde::Serialize;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use crate::error::Error;

/// Long enough to render a page of previews, short enough that a leaked link soon dies. The client
/// refetches before it runs out.
pub const PRESIGNED_URL_TTL: Duration = Duration::from_secs(15 * 60);

/// eDb's `edd.objects.file_name` and cDb's `objects.file_name` (`VARCHAR(255)`): characters.
const MAX_FILE_NAME_CHARS: usize = 255;

/// An object's eDb record, as clients see it.
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ObjectRecord {
    /// What a question's `question_objects` row stores.
    pub sha256: String,
    /// Sniffed from the bytes.
    pub content_type: String,
    /// As first uploaded, unless renamed.
    pub file_name: String,
    /// RFC 3339.
    pub created_at: String,
}

impl ObjectRecord {
    pub fn new(object: &Object) -> Result<Self, Error> {
        Ok(Self {
            sha256: object.sha256.to_string(),
            content_type: object.content_type.clone(),
            file_name: object.file_name.clone(),
            created_at: rfc3339(object.created_at)?,
        })
    }
}

/// Presigned `GET` for `key`, valid for [`PRESIGNED_URL_TTL`]. Display only: never store it.
pub async fn presigned_get(bucket: &ObjectStore, key: &str) -> Result<String, Error> {
    Ok(bucket.presigned_get(key, PRESIGNED_URL_TTL).await?)
}

/// `body` as JSON, never cached: its presigned URLs expire.
pub fn uncached_json<T: Serialize>(body: T) -> impl IntoResponse {
    (
        [(CACHE_CONTROL, HeaderValue::from_static("no-store"))],
        Json(body),
    )
}

pub fn rfc3339(at: impl Into<OffsetDateTime>) -> Result<String, Error> {
    Ok(at.into().format(&Rfc3339)?)
}

/// Shown to examiners and kept in eDb, then cDb: no control characters, at most
/// [`MAX_FILE_NAME_CHARS`]. `"unnamed"` when nothing is left.
pub fn clean_file_name(raw: &str) -> String {
    cleaned_file_name(raw).unwrap_or_else(|| "unnamed".to_string())
}

/// Like [`clean_file_name`], but `None` when nothing is left.
pub fn cleaned_file_name(raw: &str) -> Option<String> {
    let name: String = raw
        .chars()
        .filter(|c| !c.is_control())
        .take(MAX_FILE_NAME_CHARS)
        .collect();
    let name = name.trim();
    (!name.is_empty()).then(|| name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleans_file_names() {
        assert_eq!(clean_file_name(" diagram\n.png "), "diagram.png");
        assert_eq!(clean_file_name(""), "unnamed");
        assert_eq!(clean_file_name("\u{7}"), "unnamed");
        assert_eq!(cleaned_file_name(" \u{7}\n "), None);
        assert_eq!(clean_file_name(&"é".repeat(300)).chars().count(), 255);
    }
}
