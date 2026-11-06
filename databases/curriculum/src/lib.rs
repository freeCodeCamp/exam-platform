//! cDb: versioned curriculum content. Dolt.
use std::{collections::HashMap, str::FromStr, time::Duration};

use sqlx::{
    Connection, MySql, MySqlConnection, MySqlPool, Row,
    migrate::{Migrate, MigrateError, Migrator},
    mysql::{MySqlConnectOptions, MySqlPoolOptions},
    pool::PoolConnection,
};

mod aapi;
mod edd;
mod shared;

pub use edd::{Asset, NewAsset};

/// The only branch migrations reach directly.
pub const DEV: &str = "dev";
/// Where `cdb-migrate` applies migrations before merging them into [`DEV`]. Recreated from `dev`
/// on every run, so a failed run leaves `dev` untouched and nothing to clean up.
pub const SCRATCH: &str = "cdb-migrate";
/// Author of every commit `cdb-migrate` makes.
pub const AUTHOR: &str = "cdb-migrate <cdb-migrate@exam-platform.invalid>";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Migrate(#[from] MigrateError),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error("lock {0} not acquired")]
    Lock(String),
    #[error("expected branch {expected} checked out, found {actual}")]
    WrongBranch {
        expected: &'static str,
        actual: String,
    },
    #[error("merging {SCRATCH} into {DEV} conflicted ({conflicts}): aborted, {DEV} unchanged")]
    MergeConflict { conflicts: i64 },
}

/// Every migration in `migrations/`, tracked in `_sqlx_migrations` (`sqlx.toml`).
///
/// Ignores applied migrations it does not embed: those come from a newer release, and this one is a
/// rollback. Safe only because migrations stay backward compatible. Checksums of the migrations it
/// does embed are still verified.
pub fn migrator() -> Migrator {
    let mut migrator = sqlx::migrate!();
    migrator.set_ignore_missing(true);
    migrator
}

/// Connects to `url` (a database name, never a `db/branch` revision) and applies pending migrations
/// to [`DEV`], one Dolt commit each. Returns the versions applied.
pub async fn migrate(url: &str) -> Result<Vec<i64>, Error> {
    let mut conn = MySqlConnection::connect(url).await?;
    let result = migrate_with(&mut conn, &migrator()).await;
    // A failed run may still hold the lock - it goes with the session.
    let _ = conn.close().await;
    result
}

/// [`migrate`] on an open connection. Holds a named lock throughout, so concurrent runs apply each
/// migration once.
///
/// 1. Recreate [`SCRATCH`] from `dev`'s HEAD and check it out.
/// 2. Apply each pending migration there, then `dolt_commit` it. Dolt commits the SQL transaction
///    on DDL, so a migration is never atomic: a failed one is left on the scratch branch only.
/// 3. Merge into `dev`: a fast-forward unless `dev` moved meanwhile, then a merge commit.
/// 4. Delete [`SCRATCH`].
pub(crate) async fn migrate_with(
    conn: &mut MySqlConnection,
    migrator: &Migrator,
) -> Result<Vec<i64>, Error> {
    let lock = lock(conn).await?;
    let result = async {
        let applied = apply_on_scratch(conn, migrator).await?;
        if !applied.is_empty() {
            merge_into_dev(conn, &applied).await?;
        }
        drop_scratch(conn).await?;
        Ok(applied)
    }
    .await;
    let _ = unlock(conn, &lock).await;
    result
}

async fn lock(conn: &mut MySqlConnection) -> Result<String, Error> {
    let database: String = sqlx::query_scalar("SELECT DATABASE()")
        .fetch_one(&mut *conn)
        .await?;
    let name = format!("cdb-migrate:{database}");
    // Waits for as long as it takes: cdb-migrate's own timeout bounds the run.
    let acquired: Option<i64> = sqlx::query_scalar("SELECT GET_LOCK(?, -1)")
        .bind(&name)
        .fetch_one(&mut *conn)
        .await?;
    if acquired != Some(1) {
        return Err(Error::Lock(name));
    }
    Ok(name)
}

async fn unlock(conn: &mut MySqlConnection, name: &str) -> Result<(), Error> {
    sqlx::query("SELECT RELEASE_LOCK(?)")
        .bind(name)
        .execute(conn)
        .await?;
    Ok(())
}

async fn checkout(conn: &mut MySqlConnection, branch: &'static str) -> Result<(), Error> {
    sqlx::query("CALL DOLT_CHECKOUT(?)")
        .bind(branch)
        .execute(&mut *conn)
        .await?;
    let actual: String = sqlx::query_scalar("SELECT ACTIVE_BRANCH()")
        .fetch_one(&mut *conn)
        .await?;
    if actual != branch {
        return Err(Error::WrongBranch {
            expected: branch,
            actual,
        });
    }
    Ok(())
}

/// Steps 1 and 2 of [`migrate_with`]. Leaves [`SCRATCH`] checked out.
pub(crate) async fn apply_on_scratch(
    conn: &mut MySqlConnection,
    migrator: &Migrator,
) -> Result<Vec<i64>, Error> {
    // Off the scratch branch first: `-f` cannot reset the branch a session has checked out.
    checkout(conn, DEV).await?;
    sqlx::query("CALL DOLT_BRANCH('-f', ?, ?)")
        .bind(SCRATCH)
        .bind(DEV)
        .execute(&mut *conn)
        .await?;
    checkout(conn, SCRATCH).await?;

    let table = &*migrator.table_name;
    conn.ensure_migrations_table(table).await?;
    // Only a migration that failed on `dev` itself could leave one: scratch runs never reach it.
    if let Some(version) = conn.dirty_version(table).await? {
        return Err(MigrateError::Dirty(version).into());
    }
    let applied: HashMap<_, _> = conn
        .list_applied_migrations(table)
        .await?
        .into_iter()
        .map(|m| (m.version, m.checksum))
        .collect();
    if !migrator.ignore_missing
        && let Some(&version) = applied.keys().find(|v| !migrator.version_exists(**v))
    {
        return Err(MigrateError::VersionMissing(version).into());
    }

    let mut versions = Vec::new();
    for migration in migrator
        .iter()
        .filter(|m| !m.migration_type.is_down_migration())
    {
        match applied.get(&migration.version) {
            Some(checksum) if *checksum != migration.checksum => {
                return Err(MigrateError::VersionMismatch(migration.version).into());
            }
            Some(_) => {}
            None => {
                conn.apply(table, migration).await?;
                // After the migration's transaction, never inside it: a dolt_commit in an open
                // transaction can commit a stale snapshot.
                sqlx::query("CALL DOLT_COMMIT('-A', '-m', ?, '--author', ?)")
                    .bind(format!(
                        "cdb-migrate: {:04} {}",
                        migration.version, migration.description
                    ))
                    .bind(AUTHOR)
                    .execute(&mut *conn)
                    .await?;
                versions.push(migration.version);
            }
        }
    }
    Ok(versions)
}

/// Step 3 of [`migrate_with`].
pub(crate) async fn merge_into_dev(
    conn: &mut MySqlConnection,
    applied: &[i64],
) -> Result<(), Error> {
    let (first, last) = (applied[0], applied[applied.len() - 1]);
    checkout(conn, DEV).await?;
    // Fails, and rolls back, on conflicts: `dolt_allow_commit_conflicts` is off (config/server.yaml).
    let row = sqlx::query("CALL DOLT_MERGE(?, '-m', ?, '--author', ?)")
        .bind(SCRATCH)
        .bind(format!("cdb-migrate: merge {first:04}..{last:04}"))
        .bind(AUTHOR)
        .fetch_one(&mut *conn)
        .await?;
    let conflicts: i64 = row.try_get("conflicts")?;
    if conflicts != 0 {
        sqlx::query("CALL DOLT_MERGE('--abort')")
            .execute(&mut *conn)
            .await?;
        return Err(Error::MergeConflict { conflicts });
    }
    Ok(())
}

/// Step 4 of [`migrate_with`]. Leaves [`DEV`] checked out.
pub(crate) async fn drop_scratch(conn: &mut MySqlConnection) -> Result<(), Error> {
    checkout(conn, DEV).await?;
    sqlx::query("CALL DOLT_BRANCH('-D', ?)")
        .bind(SCRATCH)
        .execute(conn)
        .await?;
    Ok(())
}

#[derive(Debug, Clone)]
pub struct Database {
    pool: MySqlPool,
    /// The database `url` names, e.g. `curriculum`. Branches are `<name>/<branch>`.
    name: String,
}

impl Database {
    /// Connects to cDb. `url` names a database, never a `db/branch` revision.
    ///
    /// Never migrates. `cdb-migrate` should have already run. A `dolt_checkout` stays on its pooled
    /// connection for the next borrower, and `ROLLBACK` does not undo it: every method starts with
    /// [`Database::on_branch`], and none calls `dolt_checkout`.
    pub async fn connect(url: &str) -> Result<Self, sqlx::Error> {
        Self::connect_with(MySqlConnectOptions::from_str(url)?).await
    }

    /// Connects on first query. For tests whose requests never reach cDb.
    pub fn connect_lazy(url: &str) -> Result<Self, sqlx::Error> {
        let options = MySqlConnectOptions::from_str(url)?;
        let name = Self::name(&options)?;
        let pool = MySqlPoolOptions::new().connect_lazy_with(options);
        Ok(Self { pool, name })
    }

    fn name(options: &MySqlConnectOptions) -> Result<String, sqlx::Error> {
        options
            .get_database()
            .filter(|name| !name.is_empty() && !name.contains(['/', '`']))
            .map(str::to_owned)
            .ok_or_else(|| sqlx::Error::Configuration("cDb URL must name a database".into()))
    }

    /// [`Database::connect`], from parsed options.
    pub async fn connect_with(options: MySqlConnectOptions) -> Result<Self, sqlx::Error> {
        let name = Self::name(&options)?;
        let pool = MySqlPoolOptions::new()
            .max_connections(10)
            .acquire_timeout(Duration::from_secs(5))
            .connect_with(options)
            .await?;

        Ok(Self { pool, name })
    }

    /// A pooled connection, switched to `branch` with `USE`, which reads, writes and
    /// `dolt_commit`s there without touching checkout state.
    pub(crate) async fn on_branch(
        &self,
        branch: &str,
    ) -> Result<PoolConnection<MySql>, sqlx::Error> {
        let mut conn = self.pool.acquire().await?;
        // Neither part is user input: `name` is checked in `connect`, branches are constants.
        sqlx::raw_sql(sqlx::AssertSqlSafe(format!("USE `{}/{branch}`", self.name)))
            .execute(&mut *conn)
            .await?;
        Ok(conn)
    }
}

#[cfg(test)]
pub(crate) mod testing {
    use std::str::FromStr;

    use sqlx::{
        ConnectOptions, Connection, Executor, MySqlConnection, migrate::Migrator,
        mysql::MySqlConnectOptions,
    };

    use crate::{DEV, Error};

    /// A database of its own on the Dolt server at `CDB_MIGRATOR_URL`, with branch `dev`.
    pub struct TestDb {
        pub name: String,
        options: MySqlConnectOptions,
    }

    impl TestDb {
        /// `None` when `CDB_MIGRATOR_URL` is unset: the test then passes without running.
        pub async fn new() -> sqlx::Result<Option<Self>> {
            let Some(url) = std::env::var("CDB_MIGRATOR_URL")
                .ok()
                .filter(|v| !v.is_empty())
            else {
                eprintln!("CDB_MIGRATOR_URL not set: skipping Dolt test");
                return Ok(None);
            };
            let options = MySqlConnectOptions::from_str(&url)?;
            let name = format!("cdb_test_{}", uuid::Uuid::new_v4().simple());
            let mut conn = options.connect().await?;
            conn.execute(sqlx::AssertSqlSafe(format!("CREATE DATABASE `{name}`")))
                .await?;
            let db = Self {
                options: options.database(&name),
                name,
            };
            // The server's `init.defaultbranch` may not be `dev`.
            let mut conn = db.connect().await?;
            let has_dev: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM dolt_branches WHERE name = ?")
                    .bind(DEV)
                    .fetch_one(&mut conn)
                    .await?;
            if has_dev == 0 {
                sqlx::query("CALL DOLT_BRANCH(?)")
                    .bind(DEV)
                    .execute(&mut conn)
                    .await?;
            }
            conn.close().await?;
            Ok(Some(db))
        }

        pub async fn connect(&self) -> sqlx::Result<MySqlConnection> {
            self.options.connect().await
        }

        /// The SDK, on this database.
        pub async fn database(&self) -> sqlx::Result<crate::Database> {
            crate::Database::connect_with(self.options.clone()).await
        }

        /// Runs `migrator` as `cdb-migrate` does, on a connection of its own.
        pub async fn migrate_with(&self, migrator: &Migrator) -> Result<Vec<i64>, Error> {
            let mut conn = self.connect().await?;
            let result = crate::migrate_with(&mut conn, migrator).await;
            let _ = conn.close().await;
            result
        }

        pub async fn setup(&self) -> Result<Vec<i64>, Error> {
            self.migrate_with(&crate::migrator()).await
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

        pub async fn destroy(self) -> sqlx::Result<()> {
            let mut conn = self.connect().await?;
            conn.execute(sqlx::AssertSqlSafe(format!(
                "DROP DATABASE `{}`",
                self.name
            )))
            .await?;
            conn.close().await
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
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use sqlx::migrate::{Migration, MigrationType};

    use super::{testing::*, *};

    /// [`migrator`] plus `sql` as migration 9998.
    fn migrator_with(sql: &'static str) -> Migrator {
        let mut migrator = migrator();
        migrator.migrations.to_mut().push(Migration::new(
            9998,
            Cow::Borrowed("added by a test"),
            MigrationType::Simple,
            sqlx::SqlStr::from_static(sql),
            false,
        ));
        migrator
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

    #[test]
    fn migration_versions_are_unique() {
        let mut versions = up_versions(&migrator());
        let count = versions.len();
        versions.dedup();
        assert_eq!(versions.len(), count);
    }

    #[tokio::test]
    async fn each_migration_is_one_dolt_commit_on_dev() -> Result<(), Error> {
        let Some(db) = TestDb::new().await? else {
            return Ok(());
        };
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
        let Some(db) = TestDb::new().await? else {
            return Ok(());
        };
        db.setup().await?;
        let before = db.hash_of(DEV).await?;

        assert_eq!(db.setup().await?, Vec::<i64>::new());

        assert_eq!(db.hash_of(DEV).await?, before);
        assert!(!scratch_exists(&db).await?);
        Ok(db.destroy().await?)
    }

    /// Two runs at once: the named lock serializes them.
    #[tokio::test]
    async fn concurrent_runs_apply_each_migration_once() -> Result<(), Error> {
        let Some(db) = TestDb::new().await? else {
            return Ok(());
        };
        let migrator = migrator();

        let (a, b) = tokio::join!(db.migrate_with(&migrator), db.migrate_with(&migrator));
        let (mut a, b) = (a?, b?);

        a.extend(b);
        a.sort_unstable();
        assert_eq!(a, up_versions(&migrator));
        assert_eq!(db.applied_versions().await?, up_versions(&migrator));
        Ok(db.destroy().await?)
    }

    #[tokio::test]
    async fn older_bundle_runs_against_newer_migrations() -> Result<(), Error> {
        let Some(db) = TestDb::new().await? else {
            return Ok(());
        };
        db.setup().await?;
        apply_newer_migration(&db).await?;

        db.setup().await?;

        Ok(db.destroy().await?)
    }

    #[tokio::test]
    async fn strict_migrator_rejects_newer_migrations() -> Result<(), Error> {
        let Some(db) = TestDb::new().await? else {
            return Ok(());
        };
        db.setup().await?;
        apply_newer_migration(&db).await?;
        let mut migrator = migrator();
        migrator.set_ignore_missing(false);

        let result = db.migrate_with(&migrator).await;

        assert!(
            matches!(
                result,
                Err(Error::Migrate(MigrateError::VersionMissing(9999)))
            ),
            "{result:?}"
        );
        Ok(db.destroy().await?)
    }

    #[tokio::test]
    async fn rejects_changed_migration() -> Result<(), Error> {
        let Some(db) = TestDb::new().await? else {
            return Ok(());
        };
        db.setup().await?;
        db.commit_on_dev("UPDATE _sqlx_migrations SET checksum = X'00' WHERE version = 1")
            .await?;

        let result = db.migrate_with(&migrator()).await;

        assert!(
            matches!(
                result,
                Err(Error::Migrate(MigrateError::VersionMismatch(1)))
            ),
            "{result:?}"
        );
        Ok(db.destroy().await?)
    }

    /// The failed migration stays on the scratch branch, half applied; the next run starts over.
    #[tokio::test]
    async fn failed_migration_leaves_dev_untouched() -> Result<(), Error> {
        let Some(db) = TestDb::new().await? else {
            return Ok(());
        };
        db.setup().await?;
        let before = db.hash_of(DEV).await?;

        let result = db
            .migrate_with(&migrator_with(
                "CREATE TABLE t (id int); CREATE TABLE t (id int);",
            ))
            .await;

        assert!(result.is_err(), "{result:?}");
        assert_eq!(db.hash_of(DEV).await?, before);
        assert_eq!(
            db.migrate_with(&migrator_with("CREATE TABLE t (id int)"))
                .await?,
            vec![9998]
        );
        Ok(db.destroy().await?)
    }

    /// eDd commits content to `dev` while migrations apply: the run ends in a merge commit with both.
    #[tokio::test]
    async fn merges_when_dev_moved() -> Result<(), Error> {
        let Some(db) = TestDb::new().await? else {
            return Ok(());
        };
        let migrator = migrator();
        let mut conn = db.connect().await?;

        let applied = apply_on_scratch(&mut conn, &migrator).await?;
        db.commit_on_dev(
            "CREATE TABLE content (id int PRIMARY KEY); INSERT INTO content VALUES (1)",
        )
        .await?;
        merge_into_dev(&mut conn, &applied).await?;
        drop_scratch(&mut conn).await?;

        let message: String = sqlx::query_scalar("SELECT message FROM dolt_log(?) LIMIT 1")
            .bind(DEV)
            .fetch_one(&mut conn)
            .await?;
        let versions = up_versions(&migrator);
        assert_eq!(
            message,
            format!(
                "cdb-migrate: merge {:04}..{:04}",
                versions[0],
                versions[versions.len() - 1]
            )
        );
        let content: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM content")
            .fetch_one(&mut conn)
            .await?;
        assert_eq!(content, 1);
        conn.close().await?;
        assert_eq!(db.applied_versions().await?, up_versions(&migrator));
        Ok(db.destroy().await?)
    }

    #[tokio::test]
    async fn never_touches_other_branches() -> Result<(), Error> {
        let Some(db) = TestDb::new().await? else {
            return Ok(());
        };
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
}
