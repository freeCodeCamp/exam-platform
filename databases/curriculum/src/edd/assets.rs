//! `assets`: files questions use, recorded on [`DEV`] as eDd uploads them. The bytes are in object
//! storage at [`Asset::object_key`].
use time::PrimitiveDateTime;

use crate::{DEV, Database};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Asset {
    /// Lowercase hex SHA-256 of the bytes. Primary key.
    pub sha256: String,
    /// `sha256/<sha256>.<ext>`, in the curriculum bucket.
    pub object_key: String,
    pub content_type: String,
    pub byte_size: i32,
    pub file_name: String,
    /// UTC.
    pub created_at: PrimitiveDateTime,
}

#[derive(Debug, Clone)]
pub struct NewAsset<'a> {
    pub sha256: &'a str,
    pub object_key: &'a str,
    pub content_type: &'a str,
    pub byte_size: i32,
    pub file_name: &'a str,
}

impl Database {
    /// Records `asset` on [`DEV`] and Dolt-commits it, authored by `author` (`Name <email>`).
    /// Returns `false`, committing nothing, when an asset with the same hash is already recorded.
    ///
    /// Stages only `assets`, so other sessions' uncommitted content on `dev` stays uncommitted.
    /// Their uncommitted `assets` rows are swept in: any such row is complete on insert.
    pub async fn insert_asset(
        &self,
        asset: &NewAsset<'_>,
        author: &str,
    ) -> Result<bool, sqlx::Error> {
        let mut conn = self.on_branch(DEV).await?;

        // Autocommit: the row reaches `dev`'s working set before `dolt_commit`, which must never
        // run inside an open transaction.
        let inserted = sqlx::query!(
            "INSERT INTO assets (sha256, object_key, content_type, byte_size, file_name, created_at)
            VALUES (?, ?, ?, ?, ?, UTC_TIMESTAMP(6))",
            asset.sha256,
            asset.object_key,
            asset.content_type,
            asset.byte_size,
            asset.file_name,
        )
        .execute(&mut *conn)
        .await;
        match inserted {
            Ok(_) => {}
            Err(sqlx::Error::Database(e)) if e.is_unique_violation() => return Ok(false),
            Err(e) => return Err(e),
        }

        sqlx::query("CALL DOLT_ADD('assets')")
            .execute(&mut *conn)
            .await?;
        // `--skip-empty`: a concurrent insert's commit may already hold this row.
        sqlx::query("CALL DOLT_COMMIT('--skip-empty', '-m', ?, '--author', ?)")
            .bind(format!("eDd: add asset {}", asset.object_key))
            .bind(author)
            .execute(&mut *conn)
            .await?;

        Ok(true)
    }

    /// Every asset on [`DEV`], newest first.
    pub async fn list_assets(&self) -> Result<Vec<Asset>, sqlx::Error> {
        let mut conn = self.on_branch(DEV).await?;

        sqlx::query_as!(
            Asset,
            "SELECT sha256, object_key, content_type, byte_size, file_name, created_at
            FROM assets ORDER BY created_at DESC, sha256"
        )
        .fetch_all(&mut *conn)
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Error, testing::TestDb};

    fn new_asset<'a>(sha256: &'a str, object_key: &'a str) -> NewAsset<'a> {
        NewAsset {
            sha256,
            object_key,
            content_type: "image/png",
            byte_size: 5,
            file_name: "diagram.png",
        }
    }

    const AUTHOR: &str = "Test <test@freecodecamp.org>";

    #[tokio::test]
    async fn insert_commits_once_per_hash() -> Result<(), Error> {
        let Some(db) = TestDb::new().await? else {
            return Ok(());
        };
        db.setup().await?;
        let cdb = db.database().await?;
        let sha256 = "a".repeat(64);
        let key = format!("sha256/{sha256}.png");
        let asset = new_asset(&sha256, &key);
        let before = db.hash_of(DEV).await?;

        assert!(cdb.insert_asset(&asset, AUTHOR).await?);
        let after_insert = db.hash_of(DEV).await?;
        assert!(!cdb.insert_asset(&asset, AUTHOR).await?);

        assert_ne!(after_insert, before);
        assert_eq!(db.hash_of(DEV).await?, after_insert);
        let mut conn = db.connect().await?;
        let (message, author): (String, String) =
            sqlx::query_as("SELECT message, author FROM dolt_log(?) LIMIT 1")
                .bind(DEV)
                .fetch_one(&mut conn)
                .await?;
        assert_eq!(message, format!("eDd: add asset {key}"));
        assert_eq!(author, "Test");
        let dirty: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
            "SELECT COUNT(*) FROM `{}/{DEV}`.dolt_status",
            db.name
        )))
        .fetch_one(&mut conn)
        .await?;
        assert_eq!(dirty, 0);
        drop(conn);

        let listed = cdb.list_assets().await?;
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].sha256, sha256);
        assert_eq!(listed[0].object_key, key);
        Ok(db.destroy().await?)
    }

    /// Other content in `dev`'s working set stays out of the asset's commit.
    #[tokio::test]
    async fn commits_only_assets() -> Result<(), Error> {
        let Some(db) = TestDb::new().await? else {
            return Ok(());
        };
        db.setup().await?;
        let cdb = db.database().await?;
        let mut conn = db.connect().await?;
        sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
            "INSERT INTO `{}/{DEV}`.exams (id, slug, instructions) VALUES ('e', 'e', '')",
            db.name
        )))
        .execute(&mut conn)
        .await?;
        let sha256 = "b".repeat(64);
        let key = format!("sha256/{sha256}.png");
        let asset = new_asset(&sha256, &key);

        cdb.insert_asset(&asset, AUTHOR).await?;

        let dirty: Vec<String> = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
            "SELECT table_name FROM `{}/{DEV}`.dolt_status",
            db.name
        )))
        .fetch_all(&mut conn)
        .await?;
        assert_eq!(dirty, ["exams"]);
        Ok(db.destroy().await?)
    }
}
