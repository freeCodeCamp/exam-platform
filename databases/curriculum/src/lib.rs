//! cDb: versioned curriculum content. Dolt.
//!
//! One database, one schema per app (`edd`, `aapi`), all owned by role `cdb_migrator`.
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

/// Every migration in `migrations/`, tracked in `cdb._sqlx_migrations` (`sqlx.toml`).
///
/// Ignores applied migrations it does not embed: those come from a newer release, and this one is a
/// rollback. Safe only because migrations stay backward compatible. Checksums of the migrations it
/// does embed are still verified.
pub fn migrator() -> Migrator {
    let mut migrator = sqlx::migrate!();
    migrator.set_ignore_missing(true);
    migrator
}

/// Connects as `cdb_migrator` and applies pending migrations. Holds a Postgres advisory lock, so
/// concurrent runs from either app's release apply each migration once.
pub async fn migrate(url: &str) -> Result<(), MigrateError> {
    let lock_timeout = format!("{}s", LOCK_TIMEOUT.as_secs());
    let mut conn = PgConnectOptions::from_str(url)?
        .application_name("cdb-migrate")
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
    /// Never migrates. `cdb-migrate` should have already run.
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
                Self::Migrator => "cdb_migrator",
                Self::Edd => "edd",
                Self::Aapi => "aapi",
            }
        }
    }

    /// Bootstraps a `#[sqlx::test(migrations = false)]` database as production does: roles and
    /// schemas as superuser, then every migration as `cdb_migrator`.
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

    /// Runs `migrator` as `cdb_migrator`, on a connection of its own.
    pub async fn migrate_with(pool: &PgPool, migrator: &Migrator) -> sqlx::Result<()> {
        let mut conn = pool.acquire().await?.detach();
        let result = run_as_migrator(&mut conn, migrator).await;
        // As cdb-migrate exits: a failed run still holds the advisory lock, until its session ends.
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
