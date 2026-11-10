use std::collections::HashMap;

use sqlx::{
    Connection, MySqlConnection, Row,
    migrate::{Migrate, MigrateError, Migrator},
};

/// The only branch migrations reach directly.
pub const DEV: &str = "dev";
/// Where `cdb-migrate` applies migrations before merging them into [`DEV`]. Recreated from `dev`
/// on every run, so a failed run leaves `dev` untouched and nothing to clean up.
const SCRATCH: &str = "cdb-migrate";
/// Author of every commit `cdb-migrate` makes.
const AUTHOR: &str = "cdb-migrate <cdb-migrate@exam-platform.invalid>";

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
pub async fn migrate_with(
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
pub async fn apply_on_scratch(
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
pub async fn merge_into_dev(conn: &mut MySqlConnection, applied: &[i64]) -> Result<(), Error> {
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
pub async fn drop_scratch(conn: &mut MySqlConnection) -> Result<(), Error> {
    checkout(conn, DEV).await?;
    sqlx::query("CALL DOLT_BRANCH('-D', ?)")
        .bind(SCRATCH)
        .execute(conn)
        .await?;
    Ok(())
}
