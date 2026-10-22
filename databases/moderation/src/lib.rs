//! mDb: candidate identity, attempts, responses and evidence. Postgres.
//!
//! aAPI owns the schema: [`Database::connect`] runs the migrations in `migrations/`. eDd connects
//! as `edd` with [`Database::connect_checked`], which only verifies them. Only add, release, then
//! remove: a rolled back aAPI or eDd must still run against the newer schema.
use std::time::Duration;

use sqlx::{PgPool, migrate::Migrator, postgres::PgPoolOptions};

mod candidates;

pub use candidates::{Candidate, NewCandidate};

/// Runs every pending migration. Holds a Postgres advisory lock, so concurrent instances are safe.
///
/// Ignores applied migrations it does not embed: those come from a newer release, and this one
/// is a rollback. Safe only because migrations stay backward compatible. Checksums of the
/// migrations it does embed are still verified.
pub fn migrator() -> Migrator {
    let mut migrator = sqlx::migrate!();
    migrator.set_ignore_missing(true);
    migrator
}

#[derive(Debug, thiserror::Error)]
pub enum ConnectError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    /// The schema is older than this build: aAPI has not yet deployed the release that adds it.
    #[error("migration {0} is not applied")]
    MissingMigration(i64),
    #[error("migration {0} was applied with a different checksum")]
    ChecksumMismatch(i64),
}

#[derive(Debug, Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    /// For aAPI: connects, then runs pending migrations. Needs the owner role.
    pub async fn connect(url: &str) -> Result<Self, ConnectError> {
        let db = Self::from_pool(pool(url).await?);

        migrator().run(&db.pool).await.map_err(sqlx::Error::from)?;

        Ok(db)
    }

    /// For eDd: connects, then fails unless every embedded migration is applied. Never migrates.
    pub async fn connect_checked(url: &str) -> Result<Self, ConnectError> {
        let db = Self::from_pool(pool(url).await?);

        db.check_migrations().await?;

        Ok(db)
    }

    /// For `#[sqlx::test]`, which creates and migrates a database per test.
    pub fn from_pool(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Applied migrations this build does not embed come from a newer release: allowed.
    async fn check_migrations(&self) -> Result<(), ConnectError> {
        let applied: Vec<(i64, Vec<u8>)> =
            sqlx::query_as("SELECT version, checksum FROM _sqlx_migrations WHERE success")
                .fetch_all(&self.pool)
                .await?;

        for migration in migrator()
            .iter()
            .filter(|m| !m.migration_type.is_down_migration())
        {
            match applied
                .iter()
                .find(|(version, _)| *version == migration.version)
            {
                None => return Err(ConnectError::MissingMigration(migration.version)),
                Some((_, checksum)) if **checksum != *migration.checksum => {
                    return Err(ConnectError::ChecksumMismatch(migration.version));
                }
                Some(_) => {}
            }
        }

        Ok(())
    }
}

async fn pool(url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(5))
        .connect(url)
        .await
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use super::*;

    /// A newer release applied migration 9999, then aAPI rolled back to this release.
    async fn apply_newer_migration(pool: &PgPool) -> sqlx::Result<()> {
        sqlx::query(
            "INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time)
            VALUES (9999, 'from a newer release', true, '\\x00', 0)",
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    #[sqlx::test]
    async fn rollback_runs_against_newer_migrations(pool: PgPool) -> sqlx::Result<()> {
        apply_newer_migration(&pool).await?;

        migrator().run(&pool).await?;

        Ok(())
    }

    #[sqlx::test]
    async fn default_migrator_rejects_newer_migrations(pool: PgPool) -> sqlx::Result<()> {
        apply_newer_migration(&pool).await?;

        let result = sqlx::migrate!().run(&pool).await;

        assert!(matches!(
            result,
            Err(sqlx::migrate::MigrateError::VersionMissing(9999))
        ));
        Ok(())
    }

    #[sqlx::test]
    async fn check_accepts_newer_migrations(pool: PgPool) -> sqlx::Result<()> {
        apply_newer_migration(&pool).await?;

        assert!(Database::from_pool(pool).check_migrations().await.is_ok());
        Ok(())
    }

    #[sqlx::test]
    async fn check_rejects_missing_migration(pool: PgPool) -> sqlx::Result<()> {
        sqlx::query("DELETE FROM _sqlx_migrations WHERE version = 1")
            .execute(&pool)
            .await?;

        let result = Database::from_pool(pool).check_migrations().await;

        assert!(matches!(result, Err(ConnectError::MissingMigration(1))));
        Ok(())
    }

    #[sqlx::test]
    async fn check_rejects_changed_migration(pool: PgPool) -> sqlx::Result<()> {
        sqlx::query("UPDATE _sqlx_migrations SET checksum = '\\x00' WHERE version = 1")
            .execute(&pool)
            .await?;

        let result = Database::from_pool(pool).check_migrations().await;

        assert!(matches!(result, Err(ConnectError::ChecksumMismatch(1))));
        Ok(())
    }

    /// `edd` logs in as a member of `mdb_edd`.
    #[sqlx::test]
    async fn edd_can_check_migrations(pool: PgPool) -> sqlx::Result<()> {
        let mut tx = pool.begin().await?;
        sqlx::query("SET LOCAL ROLE mdb_edd")
            .execute(&mut *tx)
            .await?;

        let count: i64 = sqlx::query_scalar("SELECT count(*) FROM _sqlx_migrations")
            .fetch_one(&mut *tx)
            .await?;

        assert!(count > 0);
        Ok(())
    }
}
