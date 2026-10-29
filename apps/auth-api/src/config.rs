use std::env::var;

#[derive(Debug, Clone)]
pub struct AppConfig {
    /// eDb connection string. For hosted Postgres ensure `?sslmode=verify-full`.
    pub edb_database_url: String,
}

impl AppConfig {
    pub fn new() -> Self {
        let edb_database_url = match var("EDB_DATABASE_URL") {
            Ok(v) if !v.is_empty() => v,
            _ => {
                tracing::error!("EDB_DATABASE_URL not set");
                panic!("EDB_DATABASE_URL required");
            }
        };

        Self { edb_database_url }
    }
}
