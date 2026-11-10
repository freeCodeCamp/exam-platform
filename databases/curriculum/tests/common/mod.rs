use std::str::FromStr;

use sqlx::{ConnectOptions, Connection, Executor, MySqlConnection, mysql::MySqlConnectOptions};

/// A database of its own on the Dolt server at `CDB_MIGRATOR_URL`, with branch `dev`.
pub struct TestDb {
    pub name: String,
    pub(crate) options: MySqlConnectOptions,
}

impl TestDb {
    pub async fn new() -> sqlx::Result<Option<Self>> {
        let Some(url) = std::env::var("CDB_MIGRATOR_URL")
            .ok()
            .filter(|v| !v.is_empty())
        else {
            panic!(
                "CDB_MIGRATOR_URL must be set for testing.  Hint: copy the value from .env.example into .env"
            );
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
                .bind("dev")
                .fetch_one(&mut conn)
                .await?;
        if has_dev == 0 {
            sqlx::query("CALL DOLT_BRANCH(?)")
                .bind("dev")
                .execute(&mut conn)
                .await?;
        }
        conn.close().await?;
        Ok(Some(db))
    }

    pub async fn connect(&self) -> sqlx::Result<MySqlConnection> {
        self.options.connect().await
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
