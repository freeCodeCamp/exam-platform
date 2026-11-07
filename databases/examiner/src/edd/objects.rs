//! `edd.objects`: files examiners upload for questions. The bytes are in the curriculum bucket at
//! [`Object::object_key`]. An upload is not curriculum content until its row reaches cDb with the
//! content commit that first references it.
use std::fmt;

use sha2::{Digest, Sha256};
use sqlx::{PgExecutor, Postgres, Transaction};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::Database;

/// Lowercase hex SHA-256 of an object's bytes. Primary key of `edd.objects` and cDb's `objects`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, sqlx::Type)]
#[sqlx(transparent)]
pub struct ObjectHash(String);

#[derive(Debug, thiserror::Error)]
#[error("not a lowercase hex SHA-256: {0:?}")]
pub struct InvalidObjectHash(pub String);

impl ObjectHash {
    pub fn of(bytes: &[u8]) -> Self {
        Self(
            Sha256::digest(bytes)
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect(),
        )
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl TryFrom<String> for ObjectHash {
    type Error = InvalidObjectHash;

    fn try_from(hash: String) -> Result<Self, Self::Error> {
        if hash.len() == 64 && hash.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f')) {
            Ok(Self(hash))
        } else {
            Err(InvalidObjectHash(hash))
        }
    }
}

impl fmt::Display for ObjectHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// `sha256/<sha256>.<ext>`: where an object's bytes are in the curriculum bucket. The only way to
/// build one is from its [`ObjectHash`], so a key always names its bytes.
#[derive(Debug, Clone, PartialEq, Eq, Hash, sqlx::Type)]
#[sqlx(transparent)]
pub struct ObjectKey(String);

impl ObjectKey {
    /// `extension`: lowercase ASCII letters and digits, without the dot.
    ///
    /// # Panics
    ///
    /// On any other `extension`: callers pass constants.
    pub fn new(hash: &ObjectHash, extension: &str) -> Self {
        assert!(
            is_extension(extension),
            "invalid object extension {extension:?}"
        );
        Self(format!("sha256/{hash}.{extension}"))
    }

    /// `key` when it is content-addressed: `sha256/<sha256>.<ext>`, as [`ObjectKey::new`] builds.
    pub fn parse(key: &str) -> Option<Self> {
        let (hash, extension) = key.strip_prefix("sha256/")?.split_once('.')?;
        let hash = ObjectHash::try_from(hash.to_owned()).ok()?;
        is_extension(extension).then(|| Self::new(&hash, extension))
    }

    /// The hash the key names.
    pub fn hash(&self) -> ObjectHash {
        // Built only from a valid hash: 64 ASCII characters after the prefix.
        let start = "sha256/".len();
        ObjectHash(self.0[start..start + 64].to_owned())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

fn is_extension(extension: &str) -> bool {
    !extension.is_empty()
        && extension
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
}

impl fmt::Display for ObjectKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Object {
    pub sha256: ObjectHash,
    pub object_key: ObjectKey,
    pub content_type: String,
    pub byte_size: i32,
    pub file_name: String,
    /// First uploader: `edd.users.id`.
    pub uploaded_by: Uuid,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct NewObject<'a> {
    pub sha256: &'a ObjectHash,
    pub object_key: &'a ObjectKey,
    pub content_type: &'a str,
    pub byte_size: i32,
    pub file_name: &'a str,
    pub uploaded_by: Uuid,
}

/// First key of the two-key advisory lock [`ObjectKeyLock`] takes. The second is a hash of the
/// bucket key. Two-key locks never collide with sqlx's one-key migration lock.
const OBJECT_KEY_LOCK_CLASS: i32 = 0x6f62_6a73; // "objs"

/// eDd's hold on one curriculum bucket key, across every eDd instance: a transaction holding a
/// Postgres advisory lock on the key. Uploading and deleting a key both take it, so neither
/// interleaves its bucket and eDb writes with the other's. Writes through it commit with
/// [`ObjectKeyLock::commit`], which releases it. Dropped uncommitted, it rolls back and releases.
///
/// Held across bucket requests, so it holds a pool connection that long.
pub struct ObjectKeyLock(Transaction<'static, Postgres>);

impl ObjectKeyLock {
    /// The object with hash `sha256`, if any.
    pub async fn get_object(&mut self, sha256: &ObjectHash) -> Result<Option<Object>, sqlx::Error> {
        get_object(&mut *self.0, sha256).await
    }

    /// Records `object`. Returns `false`, changing nothing, when an object with the same hash is
    /// already recorded: the first upload's row stays.
    pub async fn insert_object(&mut self, object: &NewObject<'_>) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            "INSERT INTO edd.objects
                (sha256, object_key, content_type, byte_size, file_name, uploaded_by)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (sha256) DO NOTHING",
            object.sha256.as_str(),
            object.object_key.as_str(),
            object.content_type,
            object.byte_size,
            object.file_name,
            object.uploaded_by,
        )
        .execute(&mut *self.0)
        .await?;

        Ok(result.rows_affected() == 1)
    }

    /// Deletes the record of the object with hash `sha256`, not its bytes. `false` when there
    /// is none.
    pub async fn delete_object(&mut self, sha256: &ObjectHash) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!("DELETE FROM edd.objects WHERE sha256 = $1", sha256.as_str())
            .execute(&mut *self.0)
            .await?;

        Ok(result.rows_affected() == 1)
    }

    /// Commits the writes made through the lock, then releases it.
    pub async fn commit(self) -> Result<(), sqlx::Error> {
        self.0.commit().await
    }
}

async fn get_object<'e>(
    executor: impl PgExecutor<'e>,
    sha256: &ObjectHash,
) -> Result<Option<Object>, sqlx::Error> {
    sqlx::query_as!(
        Object,
        // Indented as when it was a method: the offline cache (`.sqlx/`) is keyed by the exact text.
        r#"SELECT sha256 AS "sha256: ObjectHash", object_key AS "object_key: ObjectKey",
                content_type, byte_size, file_name, uploaded_by, created_at
            FROM edd.objects WHERE sha256 = $1"#,
        sha256.as_str(),
    )
    .fetch_optional(executor)
    .await
}

impl Database {
    /// Waits for, then takes, eDd's lock on bucket key `key`. See [`ObjectKeyLock`].
    pub async fn lock_object_key(&self, key: &str) -> Result<ObjectKeyLock, sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        // Not `query!`: `pg_advisory_xact_lock` returns `void`, which the macros cannot decode.
        sqlx::query("SELECT pg_advisory_xact_lock($1, hashtext($2))")
            .bind(OBJECT_KEY_LOCK_CLASS)
            .bind(key)
            .execute(&mut *tx)
            .await?;
        Ok(ObjectKeyLock(tx))
    }

    /// The object with hash `sha256`, if any. Unlocked: to read before writing, use
    /// [`ObjectKeyLock::get_object`].
    pub async fn get_object(&self, sha256: &ObjectHash) -> Result<Option<Object>, sqlx::Error> {
        get_object(&self.pool, sha256).await
    }

    /// Sets the file name of the object with hash `sha256`. `false` when there is none.
    pub async fn rename_object(
        &self,
        sha256: &ObjectHash,
        file_name: &str,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            "UPDATE edd.objects SET file_name = $2 WHERE sha256 = $1",
            sha256.as_str(),
            file_name,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() == 1)
    }

    /// Every object, newest first.
    pub async fn list_objects(&self) -> Result<Vec<Object>, sqlx::Error> {
        sqlx::query_as!(
            Object,
            r#"SELECT sha256 AS "sha256: ObjectHash", object_key AS "object_key: ObjectKey",
                content_type, byte_size, file_name, uploaded_by, created_at
            FROM edd.objects ORDER BY created_at DESC, sha256"#
        )
        .fetch_all(&self.pool)
        .await
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use super::*;
    use crate::{NewUser, testing::*};

    async fn user_id(db: &Database, email: &str) -> sqlx::Result<Uuid> {
        let user = db
            .insert_user(NewUser {
                name: "Examiner".to_string(),
                email: email.to_string(),
                google_id: None,
                picture: None,
            })
            .await?;
        Ok(user.id)
    }

    /// A distinct object per `seed`.
    fn object(seed: &[u8]) -> (ObjectHash, ObjectKey) {
        let hash = ObjectHash::of(seed);
        let key = ObjectKey::new(&hash, "png");
        (hash, key)
    }

    fn new_object<'a>(
        hash: &'a ObjectHash,
        key: &'a ObjectKey,
        file_name: &'a str,
        uploaded_by: Uuid,
    ) -> NewObject<'a> {
        NewObject {
            sha256: hash,
            object_key: key,
            content_type: "image/png",
            byte_size: 5,
            file_name,
            uploaded_by,
        }
    }

    /// Inserts under the key's lock, as eDd does.
    async fn insert(db: &Database, object: &NewObject<'_>) -> sqlx::Result<bool> {
        let mut lock = db.lock_object_key(object.object_key.as_str()).await?;
        let inserted = lock.insert_object(object).await?;
        lock.commit().await?;
        Ok(inserted)
    }

    async fn delete(db: &Database, key: &ObjectKey) -> sqlx::Result<bool> {
        let mut lock = db.lock_object_key(key.as_str()).await?;
        let deleted = lock.delete_object(&key.hash()).await?;
        lock.commit().await?;
        Ok(deleted)
    }

    #[test]
    fn hashes_as_lowercase_hex() {
        assert_eq!(
            ObjectHash::of(b"hello").as_str(),
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn parses_only_lowercase_hex_hashes() {
        let hash = ObjectHash::of(b"hello");
        assert_eq!(ObjectHash::try_from(hash.to_string()).ok(), Some(hash));
        for bad in ["", "abc", &"A".repeat(64), &"g".repeat(64), &"a".repeat(65)] {
            assert!(ObjectHash::try_from(bad.to_owned()).is_err(), "{bad}");
        }
    }

    #[test]
    fn keys_name_their_hash() {
        let hash = ObjectHash::of(b"hello");
        assert_eq!(
            ObjectKey::new(&hash, "mp3").as_str(),
            format!("sha256/{hash}.mp3")
        );
    }

    #[test]
    fn parses_only_content_addressed_keys() {
        let hash = ObjectHash::of(b"a");
        let key = ObjectKey::parse(&format!("sha256/{hash}.png")).expect("content-addressed");
        assert_eq!(key, ObjectKey::new(&hash, "png"));
        assert_eq!(key.hash(), hash);
        for bad in [
            "stray.txt",
            "sha256/abc.png",
            &format!("sha256/{hash}"),
            &format!("sha256/{hash}."),
            &format!("sha256/{hash}.PNG"),
            &format!("sha256/{hash}.png/../x"),
            &format!("other/{hash}.png"),
        ] {
            assert_eq!(ObjectKey::parse(bad), None, "{bad}");
        }
    }

    #[test]
    #[should_panic(expected = "invalid object extension")]
    fn keys_reject_path_extensions() {
        ObjectKey::new(&ObjectHash::of(b""), "png/../x");
    }

    /// The first upload of some bytes wins: a second, by anyone, changes nothing.
    #[sqlx::test(migrations = false)]
    async fn records_each_hash_once(pool: PgPool) -> sqlx::Result<()> {
        setup(&pool).await?;
        let db = database_as(&pool, Role::Edd).await?;
        let first = user_id(&db, "a@freecodecamp.org").await?;
        let second = user_id(&db, "b@freecodecamp.org").await?;
        let (hash, key) = object(b"a");

        assert!(insert(&db, &new_object(&hash, &key, "first.png", first)).await?);
        assert!(!insert(&db, &new_object(&hash, &key, "second.png", second)).await?);

        let stored = db.get_object(&hash).await?.expect("recorded");
        assert_eq!(stored.object_key, key);
        assert_eq!(stored.file_name, "first.png");
        assert_eq!(stored.uploaded_by, first);
        assert_eq!(db.get_object(&object(b"other").0).await?, None);
        Ok(())
    }

    #[sqlx::test(migrations = false)]
    async fn lists_newest_first(pool: PgPool) -> sqlx::Result<()> {
        setup(&pool).await?;
        let db = database_as(&pool, Role::Edd).await?;
        let user = user_id(&db, "a@freecodecamp.org").await?;
        let (old, old_key) = object(b"old");
        let (new, new_key) = object(b"new");
        insert(&db, &new_object(&old, &old_key, "old.png", user)).await?;
        insert(&db, &new_object(&new, &new_key, "new.png", user)).await?;

        let listed: Vec<_> = db
            .list_objects()
            .await?
            .into_iter()
            .map(|o| o.sha256)
            .collect();

        assert_eq!(listed, [new, old]);
        Ok(())
    }

    /// The table backs up [`ObjectKey`]: a key must name its own hash.
    #[sqlx::test(migrations = false)]
    async fn rejects_keys_for_other_hashes(pool: PgPool) -> sqlx::Result<()> {
        setup(&pool).await?;
        let db = database_as(&pool, Role::Edd).await?;
        let user = user_id(&db, "a@freecodecamp.org").await?;
        let (hash, _) = object(b"a");
        let (_, other_key) = object(b"b");

        let result = insert(&db, &new_object(&hash, &other_key, "a.png", user)).await;

        let Err(sqlx::Error::Database(err)) = result else {
            panic!("expected a check violation, got {result:?}");
        };
        assert_eq!(err.constraint(), Some("objects_object_key"));
        Ok(())
    }

    #[sqlx::test(migrations = false)]
    async fn renames_and_deletes(pool: PgPool) -> sqlx::Result<()> {
        setup(&pool).await?;
        let db = database_as(&pool, Role::Edd).await?;
        let user = user_id(&db, "a@freecodecamp.org").await?;
        let (hash, key) = object(b"a");
        let (missing, _) = object(b"missing");
        insert(&db, &new_object(&hash, &key, "a.png", user)).await?;

        assert!(db.rename_object(&hash, "diagram.png").await?);

        let renamed = db.get_object(&hash).await?.map(|o| o.file_name);
        assert_eq!(renamed.as_deref(), Some("diagram.png"));
        assert!(!db.rename_object(&missing, "x.png").await?);
        assert!(delete(&db, &key).await?);
        assert!(!delete(&db, &key).await?);
        assert_eq!(db.get_object(&hash).await?, None);
        Ok(())
    }

    /// A second lock on a key waits for the first. Other keys do not.
    #[sqlx::test(migrations = false)]
    async fn locks_serialize_per_key(pool: PgPool) -> sqlx::Result<()> {
        setup(&pool).await?;
        let db = database_as(&pool, Role::Edd).await?;
        let key = "stray/a.txt";
        let wait = std::time::Duration::from_millis(200);

        let first = db.lock_object_key(key).await?;
        let other = tokio::time::timeout(wait, db.lock_object_key("stray/b.txt"))
            .await
            .expect("another key is not blocked")?;
        // Spawned, never cancelled: a cancelled wait would leave its backend queued for the lock.
        let second = tokio::spawn({
            let db = db.clone();
            async move { db.lock_object_key(key).await }
        });
        tokio::time::sleep(wait).await;
        assert!(!second.is_finished(), "the same key waits");

        first.commit().await?;
        let second = tokio::time::timeout(wait * 10, second)
            .await
            .expect("released on commit")
            .expect("lock task panicked")?;
        second.commit().await?;
        other.commit().await?;
        Ok(())
    }

    /// Dropped uncommitted, a lock's writes roll back.
    #[sqlx::test(migrations = false)]
    async fn uncommitted_writes_roll_back(pool: PgPool) -> sqlx::Result<()> {
        setup(&pool).await?;
        let db = database_as(&pool, Role::Edd).await?;
        let user = user_id(&db, "a@freecodecamp.org").await?;
        let (hash, key) = object(b"a");

        let mut lock = db.lock_object_key(key.as_str()).await?;
        lock.insert_object(&new_object(&hash, &key, "a.png", user))
            .await?;
        assert!(lock.get_object(&hash).await?.is_some());
        drop(lock);

        assert_eq!(db.get_object(&hash).await?, None);
        Ok(())
    }

    /// eDd renames, but never changes what derives from the bytes. aAPI cannot see uploads.
    #[sqlx::test(migrations = false)]
    async fn edd_changes_only_file_names_and_aapi_cannot_read(pool: PgPool) -> sqlx::Result<()> {
        setup(&pool).await?;
        for (role, sql) in [
            (Role::Edd, "UPDATE edd.objects SET content_type = 'x'"),
            (
                Role::Edd,
                "UPDATE edd.objects SET uploaded_by = uploaded_by",
            ),
            (Role::Aapi, "SELECT count(*) FROM edd.objects"),
            (Role::Aapi, "DELETE FROM edd.objects"),
        ] {
            let mut tx = pool.begin().await?;
            sqlx::query(set_local_role(role)).execute(&mut *tx).await?;

            let result = sqlx::query(sql).execute(&mut *tx).await;

            assert_denied(result);
        }
        Ok(())
    }
}
