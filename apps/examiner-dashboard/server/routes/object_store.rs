//! The object store: every object in the curriculum bucket, joined with its record in eDb's
//! `edd.objects`, so either side's strays show. Examiners rename recorded objects and delete any
//! object.
//!
//! Deleting is permanent and unchecked: eDd does not read cDb, so a cDb commit that references the
//! object keeps its row and loses the bytes. The client makes the examiner confirm that first.
//!
//! Keys travel in the `key` query parameter, never the path: a valid S3 key such as
//! `stray/../victim.txt` would be normalized by the browser into a different path.
use std::collections::HashMap;

use axum::{
    Json, Router,
    extract::{Query, State},
    response::IntoResponse,
    routing::{get, patch},
};
use edb::{Object, ObjectKey};
use http::StatusCode;
use object_storage::ListedObject;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tracing::instrument;

use crate::{
    config::ServerState,
    error::Error,
    extractor::AuthUser,
    objects::{ObjectRecord, cleaned_file_name, presigned_get, rfc3339, uncached_json},
};

pub fn router() -> Router<ServerState> {
    Router::new()
        .route("/", get(get_store))
        .route("/object", patch(patch_object).delete(delete_object))
}

#[derive(Debug, Serialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum Status {
    /// In the bucket and eDb.
    Recorded,
    /// In the bucket only, e.g. eDb failed after the bucket write. A re-upload records it.
    Unrecorded,
    /// In eDb only: the bytes are gone. Uploads skip the bucket while the record exists, so delete
    /// the record, then re-upload.
    Missing,
}

/// One key in the bucket, eDb or both.
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StoreEntry {
    /// In the curriculum bucket.
    pub key: String,
    pub status: Status,
    /// Absent when unrecorded.
    pub record: Option<ObjectRecord>,
    /// The bucket's when listed, else eDb's.
    pub byte_size: i64,
    /// RFC 3339. Absent when missing.
    pub last_modified: Option<String>,
    /// See [`presigned_get`]. Absent when missing.
    pub url: Option<String>,
}

/// One key, from either side or both.
#[derive(Debug, PartialEq, Eq)]
struct Joined {
    key: String,
    listed: Option<ListedObject>,
    recorded: Option<Object>,
}

impl Joined {
    fn status(&self) -> Status {
        match (&self.listed, &self.recorded) {
            (Some(_), Some(_)) => Status::Recorded,
            (Some(_), None) => Status::Unrecorded,
            (None, _) => Status::Missing,
        }
    }

    /// For ordering: when it reached the bucket, else when it was recorded.
    fn at(&self) -> Option<OffsetDateTime> {
        self.listed
            .as_ref()
            .and_then(|listed| listed.last_modified)
            .map(OffsetDateTime::from)
            .or_else(|| self.recorded.as_ref().map(|recorded| recorded.created_at))
    }
}

/// Joins on key, newest first, then by key.
fn join(listed: Vec<ListedObject>, recorded: Vec<Object>) -> Vec<Joined> {
    let mut recorded: HashMap<String, Object> = recorded
        .into_iter()
        .map(|object| (object.object_key.as_str().to_owned(), object))
        .collect();
    let mut joined: Vec<Joined> = listed
        .into_iter()
        .map(|listed| Joined {
            recorded: recorded.remove(&listed.key),
            key: listed.key.clone(),
            listed: Some(listed),
        })
        .collect();
    joined.extend(recorded.into_values().map(|recorded| Joined {
        key: recorded.object_key.as_str().to_owned(),
        listed: None,
        recorded: Some(recorded),
    }));
    joined.sort_by(|a, b| b.at().cmp(&a.at()).then_with(|| a.key.cmp(&b.key)));
    joined
}

impl StoreEntry {
    async fn new(joined: Joined, server_state: &ServerState) -> Result<Self, Error> {
        let status = joined.status();
        let url = match &joined.listed {
            None => None,
            Some(listed) => {
                Some(presigned_get(&server_state.curriculum_bucket, &listed.key).await?)
            }
        };
        let last_modified = joined
            .listed
            .as_ref()
            .and_then(|listed| listed.last_modified)
            .map(rfc3339)
            .transpose()?;
        let byte_size = match (&joined.listed, &joined.recorded) {
            (Some(listed), _) => listed.byte_size,
            (None, Some(recorded)) => i64::from(recorded.byte_size),
            (None, None) => unreachable!("joined from the bucket or eDb"),
        };
        Ok(Self {
            key: joined.key,
            status,
            record: joined
                .recorded
                .as_ref()
                .map(ObjectRecord::new)
                .transpose()?,
            byte_size,
            last_modified,
            url,
        })
    }
}

/// Every object in the bucket or eDb, newest first. Lists the whole bucket: one request per 1000
/// keys.
#[instrument(skip_all, err(Debug), level = "debug")]
pub async fn get_store(
    _: AuthUser,
    State(server_state): State<ServerState>,
) -> Result<impl IntoResponse, Error> {
    let listed = server_state.curriculum_bucket.list().await?;
    let recorded = server_state.database.list_objects().await?;

    let joined = join(listed, recorded);
    let mut entries = Vec::with_capacity(joined.len());
    for joined in joined {
        entries.push(StoreEntry::new(joined, &server_state).await?);
    }

    Ok(uncached_json(entries))
}

/// One bucket key, exactly as listed.
#[derive(Debug, Deserialize)]
pub struct KeyQuery {
    pub key: String,
}

impl KeyQuery {
    fn key(self) -> Result<String, Error> {
        if self.key.is_empty() {
            return Err(Error::bad_request("key is empty"));
        }
        Ok(self.key)
    }
}

/// `key`'s record in eDb, if any. Only content-addressed keys are recorded.
async fn recorded(server_state: &ServerState, key: &str) -> Result<Option<Object>, Error> {
    let Some(key) = ObjectKey::parse(key) else {
        return Ok(None);
    };
    let object = server_state.database.get_object(&key.hash()).await?;
    Ok(object.filter(|object| object.object_key == key))
}

fn not_recorded(key: &str) -> Error {
    Error::Server(StatusCode::NOT_FOUND, format!("{key} is not recorded"))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rename {
    pub file_name: String,
}

/// Renames a recorded object. cDb copies already committed keep their name.
#[instrument(skip_all, err(Debug), level = "debug")]
pub async fn patch_object(
    _: AuthUser,
    State(server_state): State<ServerState>,
    Query(query): Query<KeyQuery>,
    Json(rename): Json<Rename>,
) -> Result<impl IntoResponse, Error> {
    let key = query.key()?;
    let file_name = cleaned_file_name(&rename.file_name)
        .ok_or_else(|| Error::bad_request("file name is empty"))?;
    let object = recorded(&server_state, &key)
        .await?
        .ok_or_else(|| not_recorded(&key))?;

    let renamed = server_state
        .database
        .rename_object(&object.sha256, &file_name)
        .await?;
    if !renamed {
        return Err(not_recorded(&key));
    }

    Ok(StatusCode::NO_CONTENT)
}

/// Deletes `key`'s record and bytes. Succeeds when either or both are already gone.
///
/// Holds the key's lock throughout, as uploads do: an upload cannot record the key between the
/// record's delete and the bytes'. The record's delete commits only once the bytes are gone, so a
/// failed bucket delete leaves both. A failed commit after it leaves the record without bytes
/// (missing): delete again.
#[instrument(skip_all, err(Debug), level = "debug")]
pub async fn delete_object(
    AuthUser(user): AuthUser,
    State(server_state): State<ServerState>,
    Query(query): Query<KeyQuery>,
) -> Result<impl IntoResponse, Error> {
    let key = query.key()?;
    let mut lock = server_state.database.lock_object_key(&key).await?;
    // Only the record of this exact key: `sha256/<h>.jpg` must not delete `sha256/<h>.png`'s.
    if let Some(object_key) = ObjectKey::parse(&key) {
        let hash = object_key.hash();
        let recorded = lock.get_object(&hash).await?;
        if recorded.is_some_and(|object| object.object_key == object_key) {
            lock.delete_object(&hash).await?;
        }
    }
    server_state.curriculum_bucket.delete(&key).await?;
    lock.commit().await?;
    tracing::info!(key, examiner = user.email, "deleted object");

    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use edb::{ObjectHash, ObjectKey};
    use sqlx::types::Uuid;
    use time::Duration;

    use super::*;

    const T0: OffsetDateTime = OffsetDateTime::UNIX_EPOCH;

    fn listed(key: &str, minutes: i64) -> ListedObject {
        ListedObject {
            key: key.to_owned(),
            byte_size: 5,
            last_modified: Some((T0 + Duration::minutes(minutes)).into()),
        }
    }

    fn recorded(seed: &[u8], minutes: i64) -> Object {
        let sha256 = ObjectHash::of(seed);
        Object {
            object_key: ObjectKey::new(&sha256, "png"),
            sha256,
            content_type: "image/png".to_owned(),
            byte_size: 5,
            file_name: "a.png".to_owned(),
            uploaded_by: Uuid::nil(),
            created_at: T0 + Duration::minutes(minutes),
        }
    }

    #[test]
    fn joins_on_key_newest_first() {
        let both = recorded(b"both", 0);
        let missing = recorded(b"missing", 5);
        let both_key = both.object_key.to_string();
        let missing_key = missing.object_key.to_string();

        let joined = join(
            vec![listed(&both_key, 1), listed("stray.txt", 10)],
            vec![both, missing],
        );

        let summary: Vec<_> = joined
            .iter()
            .map(|j| (j.key.as_str(), j.status()))
            .collect();
        assert_eq!(
            summary,
            [
                ("stray.txt", Status::Unrecorded),
                (missing_key.as_str(), Status::Missing),
                (both_key.as_str(), Status::Recorded),
            ]
        );
    }
}
