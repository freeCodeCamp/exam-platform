//! `objects`: files questions use, recorded on [`DEV`] as eDd uploads them. The bytes are in object
//! storage at [`Object::object_key`].
use std::fmt;

use sha2::{Digest, Sha256};
use time::PrimitiveDateTime;

use crate::{DEV, Database, Error, acquire_lock, merge_into_dev};

/// Where [`Database::insert_object`] commits each object before merging it into [`DEV`]. Recreated
/// from `dev` on every insert.
pub(crate) const OBJECTS_BRANCH: &str = "edd-objects";
/// Seconds [`Database::insert_object`] waits for another insert to finish.
const LOCK_TIMEOUT_SECS: i64 = 30;

/// Lowercase hex SHA-256 of an object's bytes. Primary key of `objects`.
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
            !extension.is_empty()
                && extension
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit()),
            "invalid object extension {extension:?}"
        );
        Self(format!("sha256/{hash}.{extension}"))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
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
    /// UTC.
    pub created_at: PrimitiveDateTime,
}

#[derive(Debug, Clone)]
pub struct NewObject<'a> {
    pub sha256: &'a ObjectHash,
    pub object_key: &'a ObjectKey,
    pub content_type: &'a str,
    pub byte_size: i32,
    pub file_name: &'a str,
}

impl Database {
    /// Records `object` on [`DEV`] in one Dolt commit, authored by `author` (`Name <email>`).
    /// Returns `false`, committing nothing, when `dev`'s HEAD already holds an object with the same
    /// hash.
    ///
    /// Never touches `dev`'s working set until the commit exists: under a named lock,
    /// 1. Recreate [`OBJECTS_BRANCH`] from `dev`'s HEAD.
    /// 2. Insert the row there and commit it.
    /// 3. Merge into `dev` with [`merge_into_dev`].
    ///
    /// A failure at any step leaves `dev` without the row, so a retry redoes every step. The merge
    /// builds on `dev`'s HEAD, never its working set: other sessions' uncommitted or staged content
    /// stays out of the commit and stays in the working set, unstaged. Fails while `dev`'s working
    /// set has uncommitted `objects` changes: the merge would overwrite them.
    pub async fn insert_object(&self, object: &NewObject<'_>, author: &str) -> Result<bool, Error> {
        let mut conn = self.on_branch(DEV).await?;
        // The lock and `USE` go with the session, even when this future is dropped part way.
        conn.close_on_drop();
        acquire_lock(
            &mut conn,
            &format!("edd-objects:{}", self.name),
            LOCK_TIMEOUT_SECS,
        )
        .await?;

        sqlx::query("CALL DOLT_BRANCH('-f', ?, ?)")
            .bind(OBJECTS_BRANCH)
            .bind(DEV)
            .execute(&mut *conn)
            .await?;
        self.switch(&mut conn, OBJECTS_BRANCH).await?;

        // Autocommit: `dolt_commit` must never run inside an open transaction.
        let inserted = sqlx::query!(
            "INSERT INTO objects (sha256, object_key, content_type, byte_size, file_name, created_at)
            VALUES (?, ?, ?, ?, ?, UTC_TIMESTAMP(6))",
            object.sha256.as_str(),
            object.object_key.as_str(),
            object.content_type,
            object.byte_size,
            object.file_name,
        )
        .execute(&mut *conn)
        .await;
        match inserted {
            Ok(_) => {}
            Err(sqlx::Error::Database(e)) if e.is_unique_violation() => return Ok(false),
            Err(e) => return Err(e.into()),
        }
        sqlx::query("CALL DOLT_COMMIT('-A', '-m', ?, '--author', ?)")
            .bind(format!("eDd: add object {}", object.object_key))
            .bind(author)
            .execute(&mut *conn)
            .await?;

        self.switch(&mut conn, DEV).await?;
        merge_into_dev(
            &mut conn,
            OBJECTS_BRANCH,
            &format!("eDd: merge object {}", object.object_key),
            author,
        )
        .await?;

        Ok(true)
    }

    /// The object on [`DEV`] with hash `sha256`, if any.
    pub async fn get_object(&self, sha256: &ObjectHash) -> Result<Option<Object>, sqlx::Error> {
        let mut conn = self.on_branch(DEV).await?;

        sqlx::query_as!(
            Object,
            r#"SELECT sha256 AS "sha256: ObjectHash", object_key AS "object_key: ObjectKey",
                content_type, byte_size, file_name, created_at
            FROM objects WHERE sha256 = ?"#,
            sha256.as_str(),
        )
        .fetch_optional(&mut *conn)
        .await
    }

    /// Every object on [`DEV`], newest first.
    pub async fn list_objects(&self) -> Result<Vec<Object>, sqlx::Error> {
        let mut conn = self.on_branch(DEV).await?;

        sqlx::query_as!(
            Object,
            r#"SELECT sha256 AS "sha256: ObjectHash", object_key AS "object_key: ObjectKey",
                content_type, byte_size, file_name, created_at
            FROM objects ORDER BY created_at DESC, sha256"#
        )
        .fetch_all(&mut *conn)
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::TestDb;

    const AUTHOR: &str = "Test <test@freecodecamp.org>";

    /// A distinct object per `seed`.
    fn object(seed: &[u8]) -> (ObjectHash, ObjectKey) {
        let hash = ObjectHash::of(seed);
        let key = ObjectKey::new(&hash, "png");
        (hash, key)
    }

    fn new_object<'a>(hash: &'a ObjectHash, key: &'a ObjectKey) -> NewObject<'a> {
        NewObject {
            sha256: hash,
            object_key: key,
            content_type: "image/png",
            byte_size: 5,
            file_name: "diagram.png",
        }
    }

    /// Runs `sql` on `branch`, without committing.
    async fn run_on(db: &TestDb, branch: &str, sql: &str) -> sqlx::Result<()> {
        let mut conn = db.connect().await?;
        sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
            "USE `{}/{branch}`; {sql}",
            db.name
        )))
        .execute(&mut conn)
        .await?;
        Ok(())
    }

    fn insert_sql(hash: &ObjectHash, key: &ObjectKey) -> String {
        format!(
            "INSERT INTO objects (sha256, object_key, content_type, byte_size, file_name, created_at)
            VALUES ('{hash}', '{key}', 'image/png', 5, 'x.png', UTC_TIMESTAMP(6));"
        )
    }

    async fn count_on_head(db: &TestDb, table: &str) -> sqlx::Result<i64> {
        let mut conn = db.connect().await?;
        sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
            "SELECT COUNT(*) FROM `{}/{DEV}`.{table} AS OF 'HEAD'",
            db.name
        )))
        .fetch_one(&mut conn)
        .await
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
    #[should_panic(expected = "invalid object extension")]
    fn keys_reject_path_extensions() {
        ObjectKey::new(&ObjectHash::of(b""), "png/../x");
    }

    #[tokio::test]
    async fn insert_commits_once_per_hash() -> Result<(), Error> {
        let Some(db) = TestDb::new().await? else {
            return Ok(());
        };
        db.setup().await?;
        let cdb = db.database().await?;
        let (hash, key) = object(b"a");
        let before = db.hash_of(DEV).await?;

        assert!(cdb.insert_object(&new_object(&hash, &key), AUTHOR).await?);
        let after_insert = db.hash_of(DEV).await?;
        assert!(!cdb.insert_object(&new_object(&hash, &key), AUTHOR).await?);

        assert_ne!(after_insert, before);
        assert_eq!(db.hash_of(DEV).await?, after_insert);
        let mut conn = db.connect().await?;
        let (message, author): (String, String) =
            sqlx::query_as("SELECT message, author FROM dolt_log(?) LIMIT 1")
                .bind(DEV)
                .fetch_one(&mut conn)
                .await?;
        assert_eq!(message, format!("eDd: add object {key}"));
        assert_eq!(author, "Test");
        let dirty: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
            "SELECT COUNT(*) FROM `{}/{DEV}`.dolt_status",
            db.name
        )))
        .fetch_one(&mut conn)
        .await?;
        assert_eq!(dirty, 0);
        drop(conn);

        let listed = cdb.list_objects().await?;
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].sha256, hash);
        assert_eq!(listed[0].object_key, key);
        assert_eq!(cdb.get_object(&hash).await?.as_ref(), Some(&listed[0]));
        assert_eq!(cdb.get_object(&object(b"other").0).await?, None);
        Ok(db.destroy().await?)
    }

    /// Other sessions' content in `dev`'s working set, staged or not, stays out of the object's
    /// commit and stays in the working set.
    #[tokio::test]
    async fn commits_only_objects() -> Result<(), Error> {
        let Some(db) = TestDb::new().await? else {
            return Ok(());
        };
        db.setup().await?;
        let cdb = db.database().await?;
        run_on(
            &db,
            DEV,
            "INSERT INTO exams (id, slug, instructions) VALUES ('e', 'e', '');
            CALL DOLT_ADD('exams');",
        )
        .await?;
        let (hash, key) = object(b"b");

        assert!(cdb.insert_object(&new_object(&hash, &key), AUTHOR).await?);

        assert_eq!(count_on_head(&db, "objects").await?, 1);
        assert_eq!(count_on_head(&db, "exams").await?, 0);
        let mut conn = db.connect().await?;
        let dirty: Vec<String> = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
            "SELECT table_name FROM `{}/{DEV}`.dolt_status",
            db.name
        )))
        .fetch_all(&mut conn)
        .await?;
        assert_eq!(dirty, ["exams"]);
        let exams: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
            "SELECT COUNT(*) FROM `{}/{DEV}`.exams",
            db.name
        )))
        .fetch_one(&mut conn)
        .await?;
        assert_eq!(exams, 1);
        drop(conn);
        Ok(db.destroy().await?)
    }

    /// A run that committed on [`OBJECTS_BRANCH`] but never merged leaves `dev` without the row,
    /// so the retry commits it.
    #[tokio::test]
    async fn retry_completes_unmerged_insert() -> Result<(), Error> {
        let Some(db) = TestDb::new().await? else {
            return Ok(());
        };
        db.setup().await?;
        let cdb = db.database().await?;
        let (hash, key) = object(b"c");
        run_on(
            &db,
            DEV,
            &format!("CALL DOLT_BRANCH('{OBJECTS_BRANCH}', '{DEV}')"),
        )
        .await?;
        run_on(
            &db,
            OBJECTS_BRANCH,
            &format!(
                "{} CALL DOLT_COMMIT('-A', '-m', 'interrupted');",
                insert_sql(&hash, &key)
            ),
        )
        .await?;

        assert!(cdb.insert_object(&new_object(&hash, &key), AUTHOR).await?);

        assert_eq!(count_on_head(&db, "objects").await?, 1);
        assert!(!cdb.insert_object(&new_object(&hash, &key), AUTHOR).await?);
        Ok(db.destroy().await?)
    }

    /// Uncommitted `objects` rows on `dev` block the merge, leaving `dev` as it was.
    #[tokio::test]
    async fn uncommitted_objects_on_dev_fail_the_insert() -> Result<(), Error> {
        let Some(db) = TestDb::new().await? else {
            return Ok(());
        };
        db.setup().await?;
        let cdb = db.database().await?;
        let (other_hash, other_key) = object(b"d");
        run_on(&db, DEV, &insert_sql(&other_hash, &other_key)).await?;
        let before = db.hash_of(DEV).await?;
        let (hash, key) = object(b"e");

        assert!(
            cdb.insert_object(&new_object(&hash, &key), AUTHOR)
                .await
                .is_err()
        );

        assert_eq!(db.hash_of(DEV).await?, before);
        assert_eq!(count_on_head(&db, "objects").await?, 0);
        Ok(db.destroy().await?)
    }

    /// Concurrent inserts take turns: each commits its own row, and the same hash commits once.
    #[tokio::test]
    async fn concurrent_inserts_each_commit() -> Result<(), Error> {
        let Some(db) = TestDb::new().await? else {
            return Ok(());
        };
        db.setup().await?;
        let cdb = db.database().await?;
        let (f, fk) = object(b"f");
        let (g, gk) = object(b"g");
        let (a, b) = (new_object(&f, &fk), new_object(&g, &gk));

        let results = tokio::join!(
            cdb.insert_object(&a, AUTHOR),
            cdb.insert_object(&b, AUTHOR),
            cdb.insert_object(&a, AUTHOR),
        );

        let inserted = [results.0?, results.1?, results.2?];
        assert_eq!(inserted.iter().filter(|i| **i).count(), 2);
        assert!(inserted[1]);
        assert_eq!(count_on_head(&db, "objects").await?, 2);
        Ok(db.destroy().await?)
    }
}
