//! cDb: versioned curriculum content. Dolt.
use std::time::Duration;

use sqlx::{MySqlPool, mysql::MySqlPoolOptions};

mod aapi;
pub mod edd;
pub mod migrations;
mod shared;

#[derive(Debug, Clone)]
pub struct Database {
    pool: MySqlPool,
}

impl Database {
    /// Connects to cDb.
    ///
    /// Never migrates. `cdb-migrate` should have already run. A `dolt_checkout` stays on its pooled
    /// connection for the next borrower, and `ROLLBACK` does not undo it: start every transaction with
    /// `` USE `curriculum/<branch>` ``, or give each branch a pool that runs that `USE` in
    /// `after_connect` and never calls `dolt_checkout`.
    pub async fn connect(url: &str) -> Result<Self, sqlx::Error> {
        let pool = MySqlPoolOptions::new()
            .max_connections(10)
            .acquire_timeout(Duration::from_secs(5))
            .connect(url)
            .await?;

        Ok(Self { pool })
    }

    pub fn from_pool(pool: MySqlPool) -> Self {
        Self { pool }
    }
}
