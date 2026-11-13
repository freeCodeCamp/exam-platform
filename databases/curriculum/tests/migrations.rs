mod common;

use cdb::migrations::{self, DEV, Error, migrator};
use common::TestDb;
use sqlx::{ConnectOptions, Connection, migrate::Migrator, mysql::MySqlConnectOptions};

const SCRATCH: &str = "cdb-migrate";

impl TestDb {
    pub async fn setup(&self) -> Result<Vec<i64>, Error> {
        let url = MySqlConnectOptions::to_url_lossy(&self.options);
        migrations::migrate(url.as_str()).await
    }

    /// Runs `sql` on `dev` and commits it there, as eDd would.
    pub async fn commit_on_dev(&self, sql: &'static str) -> sqlx::Result<()> {
        let mut conn = self.connect().await?;
        sqlx::query("CALL DOLT_CHECKOUT(?)")
            .bind(DEV)
            .execute(&mut conn)
            .await?;
        sqlx::raw_sql(sql).execute(&mut conn).await?;
        sqlx::query("CALL DOLT_COMMIT('-A', '-m', 'test: on dev')")
            .execute(&mut conn)
            .await?;
        conn.close().await
    }

    pub async fn hash_of(&self, branch: &str) -> sqlx::Result<String> {
        let mut conn = self.connect().await?;
        let hash = sqlx::query_scalar("SELECT HASHOF(?)")
            .bind(branch)
            .fetch_one(&mut conn)
            .await?;
        conn.close().await?;
        Ok(hash)
    }

    /// Successful migrations recorded on `dev`.
    pub async fn applied_versions(&self) -> sqlx::Result<Vec<i64>> {
        let mut conn = self.connect().await?;
        let versions = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
            "SELECT version FROM `{}/{DEV}`._sqlx_migrations WHERE success ORDER BY version",
            self.name
        )))
        .fetch_all(&mut conn)
        .await?;
        conn.close().await?;
        Ok(versions)
    }
}

/// Versions of every up migration `migrator` embeds.
pub fn up_versions(migrator: &Migrator) -> Vec<i64> {
    migrator
        .iter()
        .filter(|m| !m.migration_type.is_down_migration())
        .map(|m| m.version)
        .collect()
}

/// A newer release applied migration 9999 on `dev`, then rolled back to this release.
async fn apply_newer_migration(db: &TestDb) -> sqlx::Result<()> {
    db.commit_on_dev(
        "INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time)
        VALUES (9999, 'from a newer release', true, X'00', 0)",
    )
    .await
}

async fn scratch_exists(db: &TestDb) -> sqlx::Result<bool> {
    let mut conn = db.connect().await?;
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM dolt_branches WHERE name = ?")
        .bind(SCRATCH)
        .fetch_one(&mut conn)
        .await?;
    conn.close().await?;
    Ok(count > 0)
}

#[tokio::test]
async fn each_migration_is_one_dolt_commit_on_dev() -> Result<(), Error> {
    let db = TestDb::new().await?.unwrap();
    let expected = up_versions(&migrator());

    assert_eq!(db.setup().await?, expected);

    let mut conn = db.connect().await?;
    let log: Vec<(String, String)> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "SELECT message, author FROM dolt_log('{DEV}') LIMIT {}",
        expected.len()
    )))
    .fetch_all(&mut conn)
    .await?;
    let messages: Vec<_> = migrator()
        .iter()
        .rev()
        .map(|m| format!("cdb-migrate: {:04} {}", m.version, m.description))
        .collect();
    assert_eq!(
        log.iter().map(|(m, _)| m).collect::<Vec<_>>(),
        messages.iter().collect::<Vec<_>>()
    );
    assert!(log.iter().all(|(_, c)| c == "cdb-migrate"), "{log:?}");
    let dirty: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
        "SELECT COUNT(*) FROM `{}/{DEV}`.dolt_status",
        db.name
    )))
    .fetch_one(&mut conn)
    .await?;
    assert_eq!(dirty, 0);
    conn.close().await?;
    assert_eq!(db.applied_versions().await?, expected);
    assert!(!scratch_exists(&db).await?);
    Ok(db.destroy().await?)
}

#[tokio::test]
async fn rerun_is_a_no_op() -> Result<(), Error> {
    let db = TestDb::new().await?.unwrap();
    db.setup().await?;
    let before = db.hash_of(DEV).await?;

    assert_eq!(db.setup().await?, Vec::<i64>::new());

    assert_eq!(db.hash_of(DEV).await?, before);
    assert!(!scratch_exists(&db).await?);
    Ok(db.destroy().await?)
}

#[tokio::test]
async fn older_bundle_runs_against_newer_migrations() -> Result<(), Error> {
    let db = TestDb::new().await?.unwrap();
    db.setup().await?;
    apply_newer_migration(&db).await?;

    db.setup().await?;

    Ok(db.destroy().await?)
}

#[tokio::test]
async fn never_touches_other_branches() -> Result<(), Error> {
    let db = TestDb::new().await?.unwrap();
    let mut conn = db.connect().await?;
    sqlx::query("CALL DOLT_BRANCH('prd', ?)")
        .bind(DEV)
        .execute(&mut conn)
        .await?;
    conn.close().await?;
    let before = db.hash_of("prd").await?;

    db.setup().await?;

    assert_eq!(db.hash_of("prd").await?, before);
    Ok(db.destroy().await?)
}
