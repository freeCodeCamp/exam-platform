//! eDb: everything eDd owns that is not versioned curriculum content. Postgres.
//!
//! Migrations in `migrations/` are embedded and run by [`Database::connect`]. Only add, release,
//! then remove: a rolled back eDd must still run against the newer schema.
use std::time::Duration;

use sqlx::{PgPool, migrate::Migrator, postgres::PgPoolOptions};

mod sessions;
mod users;

pub use sessions::Session;
pub use users::{NewUser, User};

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

#[derive(Debug, Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    /// Connects, then runs pending migrations.
    pub async fn connect(url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .acquire_timeout(Duration::from_secs(5))
            .connect(url)
            .await?;

        migrator().run(&pool).await?;

        Ok(Self { pool })
    }

    /// For `#[sqlx::test]`, which creates and migrates a database per test.
    pub fn from_pool(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use super::*;

    /// A newer release applied migration 9999, then eDd rolled back to this release.
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
}
