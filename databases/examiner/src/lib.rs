//! eDb: everything eDd and aAPI keep that is not versioned curriculum content. Postgres.
//!
//! One database, one schema per app (`edd`, `aapi`), all owned by role `edb_migrator`
//! (`initdb/10-bootstrap.sql`). Only `edb-migrate` (`src/bin/edb-migrate.rs`) changes the schema:
//! deploys run it before replacing either app. The apps connect as `edd` and `aapi`, with only the
//! table grants `migrations/` gives them, and never migrate. Only add, release, then remove: a
//! rolled back eDd or aAPI must still run against the newer schema.
use std::{str::FromStr, time::Duration};

use sqlx::{
    ConnectOptions, PgPool,
    migrate::{MigrateError, Migrator},
    postgres::{PgConnectOptions, PgPoolOptions},
};

mod aapi;
mod edd;
mod shared;

pub use aapi::{Candidate, NewCandidate};
pub use edd::{NewUser, Session, User};

/// How long a migration statement waits for a lock
pub const LOCK_TIMEOUT: Duration = Duration::from_secs(60);

/// Every migration in `migrations/`, tracked in `edb._sqlx_migrations` (`sqlx.toml`).
///
/// Ignores applied migrations it does not embed: those come from a newer release, and this one is a
/// rollback. Safe only because migrations stay backward compatible. Checksums of the migrations it
/// does embed are still verified.
pub fn migrator() -> Migrator {
    let mut migrator = sqlx::migrate!();
    migrator.set_ignore_missing(true);
    migrator
}

/// Connects as `edb_migrator` and applies pending migrations. Holds a Postgres advisory lock, so
/// concurrent runs from either app's release apply each migration once.
pub async fn migrate(url: &str) -> Result<(), MigrateError> {
    let lock_timeout = format!("{}s", LOCK_TIMEOUT.as_secs());
    let mut conn = PgConnectOptions::from_str(url)?
        .application_name("edb-migrate")
        .options([("lock_timeout", lock_timeout.as_str())])
        .connect()
        .await?;
    migrator().run(&mut conn).await
}

#[derive(Debug, Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    /// Connects as an app's role.
    /// 
    /// Never migrates. `edb-migrate` should have already run.
    pub async fn connect(url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .acquire_timeout(Duration::from_secs(5))
            .connect(url)
            .await?;

        Ok(Self { pool })
    }

    /// For `#[sqlx::test(migrations = false)]`, after `testing::setup`.
    pub fn from_pool(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[cfg(test)]
pub(crate) mod testing {
    use sqlx::{Connection, PgConnection, PgPool, migrate::Migrator, postgres::PgPoolOptions};

    use crate::Database;

    /// A login role from `initdb/10-bootstrap.sql`.
    #[derive(Debug, Clone, Copy)]
    pub enum Role {
        Migrator,
        Edd,
        Aapi,
    }

    impl Role {
        fn name(self) -> &'static str {
            match self {
                Self::Migrator => "edb_migrator",
                Self::Edd => "edd",
                Self::Aapi => "aapi",
            }
        }
    }

    /// Bootstraps a `#[sqlx::test(migrations = false)]` database as production does: roles and
    /// schemas as superuser, then every migration as `edb_migrator`.
    pub async fn setup(pool: &PgPool) -> sqlx::Result<()> {
        bootstrap(pool).await?;
        migrate_with(pool, &super::migrator()).await?;
        Ok(())
    }

    /// A [`Database`] on `pool`'s test database, as `role`: what the app itself can do.
    pub async fn database_as(pool: &PgPool, role: Role) -> sqlx::Result<Database> {
        let pool = PgPoolOptions::new()
            .after_connect(move |conn, _| {
                Box::pin(async move {
                    sqlx::query(set_role(role)).execute(conn).await?;
                    Ok(())
                })
            })
            .connect_with((*pool.connect_options()).clone())
            .await?;
        Ok(Database::from_pool(pool))
    }

    pub async fn bootstrap(pool: &PgPool) -> sqlx::Result<()> {
        sqlx::raw_sql(include_str!("../initdb/10-bootstrap.sql"))
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Runs `migrator` as `edb_migrator`, on a connection of its own.
    pub async fn migrate_with(pool: &PgPool, migrator: &Migrator) -> sqlx::Result<()> {
        let mut conn = pool.acquire().await?.detach();
        let result = run_as_migrator(&mut conn, migrator).await;
        // As edb-migrate exits: a failed run still holds the advisory lock, until its session ends.
        conn.close().await?;
        result
    }

    async fn run_as_migrator(conn: &mut PgConnection, migrator: &Migrator) -> sqlx::Result<()> {
        sqlx::query(set_role(Role::Migrator))
            .execute(&mut *conn)
            .await?;
        migrator.run(&mut *conn).await?;
        Ok(())
    }

    pub fn set_role(role: Role) -> sqlx::AssertSqlSafe<String> {
        sqlx::AssertSqlSafe(format!("SET ROLE {}", role.name()))
    }

    /// Like [`set_role`], for a transaction.
    pub fn set_local_role(role: Role) -> sqlx::AssertSqlSafe<String> {
        sqlx::AssertSqlSafe(format!("SET LOCAL ROLE {}", role.name()))
    }

    /// Asserts `result` failed for lack of a privilege.
    pub fn assert_denied<T: std::fmt::Debug>(result: sqlx::Result<T>) {
        let Err(sqlx::Error::Database(err)) = result else {
            panic!("expected permission denied, got {result:?}");
        };
        assert_eq!(err.code().as_deref(), Some("42501"), "{err}");
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use sqlx::{
        PgPool,
        migrate::{Migration, MigrationType},
    };

    use super::{testing::*, *};

    /// A newer release applied migration 9999, then rolled back to this release.
    async fn apply_newer_migration(pool: &PgPool) -> sqlx::Result<()> {
        sqlx::query(
            "INSERT INTO edb._sqlx_migrations (version, description, success, checksum, execution_time)
            VALUES (9999, 'from a newer release', true, '\\x00', 0)",
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    /// [`migrator`] plus `sql` as migration 9998.
    fn migrator_with(sql: &'static str) -> Migrator {
        let mut migrator = migrator();
        migrator.migrations.to_mut().push(Migration::new(
            9998,
            Cow::Borrowed("added by a test"),
            MigrationType::ReversibleUp,
            sqlx::SqlStr::from_static(sql),
            false,
        ));
        migrator
    }

    async fn applied_versions(pool: &PgPool) -> sqlx::Result<Vec<i64>> {
        sqlx::query_scalar(
            "SELECT version FROM edb._sqlx_migrations WHERE success ORDER BY version",
        )
        .fetch_all(pool)
        .await
    }

    #[test]
    fn migration_versions_are_unique() {
        let mut versions: Vec<_> = migrator()
            .iter()
            .filter(|m| !m.migration_type.is_down_migration())
            .map(|m| m.version)
            .collect();
        let count = versions.len();
        versions.dedup();
        assert_eq!(versions.len(), count);
    }

    #[sqlx::test(migrations = false)]
    async fn rerun_is_a_no_op(pool: PgPool) -> sqlx::Result<()> {
        setup(&pool).await?;
        let before = applied_versions(&pool).await?;

        migrate_with(&pool, &migrator()).await?;

        assert_eq!(applied_versions(&pool).await?, before);
        Ok(())
    }

    /// Both backends' releases migrate at once: the advisory lock serializes them.
    #[sqlx::test(migrations = false)]
    async fn concurrent_runs_apply_each_migration_once(pool: PgPool) -> sqlx::Result<()> {
        bootstrap(&pool).await?;
        let migrator = migrator();

        let (a, b) = tokio::join!(
            migrate_with(&pool, &migrator),
            migrate_with(&pool, &migrator)
        );
        a?;
        b?;

        let expected: Vec<_> = migrator
            .iter()
            .filter(|m| !m.migration_type.is_down_migration())
            .map(|m| m.version)
            .collect();
        assert_eq!(applied_versions(&pool).await?, expected);
        Ok(())
    }

    #[sqlx::test(migrations = false)]
    async fn older_bundle_runs_against_newer_migrations(pool: PgPool) -> sqlx::Result<()> {
        setup(&pool).await?;
        apply_newer_migration(&pool).await?;

        migrate_with(&pool, &migrator()).await?;

        Ok(())
    }

    #[sqlx::test(migrations = false)]
    async fn strict_migrator_rejects_newer_migrations(pool: PgPool) -> sqlx::Result<()> {
        setup(&pool).await?;
        apply_newer_migration(&pool).await?;
        let mut migrator = migrator();
        migrator.set_ignore_missing(false);

        let result = migrate_with(&pool, &migrator).await;

        assert!(
            matches!(
                result,
                Err(sqlx::Error::Migrate(ref err)) if matches!(**err, MigrateError::VersionMissing(9999))
            ),
            "{result:?}"
        );
        Ok(())
    }

    #[sqlx::test(migrations = false)]
    async fn rejects_changed_migration(pool: PgPool) -> sqlx::Result<()> {
        setup(&pool).await?;
        sqlx::query("UPDATE edb._sqlx_migrations SET checksum = '\\x00' WHERE version = 1")
            .execute(&pool)
            .await?;

        let result = migrate_with(&pool, &migrator()).await;

        assert!(
            matches!(
                result,
                Err(sqlx::Error::Migrate(ref err)) if matches!(**err, MigrateError::VersionMismatch(1))
            ),
            "{result:?}"
        );
        Ok(())
    }

    /// Earlier migrations stay applied; the failed one rolls back and runs again next time.
    #[sqlx::test(migrations = false)]
    async fn failed_migration_keeps_earlier_ones(pool: PgPool) -> sqlx::Result<()> {
        bootstrap(&pool).await?;
        let before: Vec<_> = migrator()
            .iter()
            .filter(|m| !m.migration_type.is_down_migration())
            .map(|m| m.version)
            .collect();

        let result = migrate_with(
            &pool,
            &migrator_with("CREATE TABLE edd.t (id int); SELECT 1 / 0;"),
        )
        .await;

        assert!(result.is_err());
        assert_eq!(applied_versions(&pool).await?, before);
        let table: Option<String> = sqlx::query_scalar("SELECT to_regclass('edd.t')::text")
            .fetch_one(&pool)
            .await?;
        assert_eq!(table, None);
        migrate_with(&pool, &migrator_with("CREATE TABLE edd.t (id int)")).await?;
        Ok(())
    }

    /// Migrations own every object, so the app roles cannot change any.
    #[sqlx::test(migrations = false)]
    async fn app_roles_cannot_change_the_schema(pool: PgPool) -> sqlx::Result<()> {
        setup(&pool).await?;
        for role in [Role::Edd, Role::Aapi] {
            for sql in [
                "CREATE TABLE edd.t (id int)",
                "CREATE TABLE aapi.t (id int)",
                "CREATE TABLE public.t (id int)",
                "CREATE TEMP TABLE t (id int)",
                "CREATE SCHEMA t",
                "ALTER TABLE edd.users ADD COLUMN t int",
                "ALTER TABLE aapi.candidates ADD COLUMN t int",
                "DROP TABLE edd.users",
                "DROP TABLE aapi.candidates",
                "CREATE INDEX t ON aapi.candidates (name)",
                "TRUNCATE aapi.candidates",
            ] {
                let mut tx = pool.begin().await?;
                sqlx::query(set_local_role(role)).execute(&mut *tx).await?;

                let result = sqlx::query(sql).execute(&mut *tx).await;

                // Not owning a table is 42501 too.
                assert_denied(result);
            }
        }
        Ok(())
    }

    #[sqlx::test(migrations = false)]
    async fn app_roles_cannot_read_migration_history(pool: PgPool) -> sqlx::Result<()> {
        setup(&pool).await?;
        for role in [Role::Edd, Role::Aapi] {
            let mut tx = pool.begin().await?;
            sqlx::query(set_local_role(role)).execute(&mut *tx).await?;

            let result = sqlx::query("SELECT count(*) FROM edb._sqlx_migrations")
                .execute(&mut *tx)
                .await;

            assert_denied(result);
        }
        Ok(())
    }
}
