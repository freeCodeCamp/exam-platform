use std::env::var;

use axum::extract::FromRef;
use axum_extra::extract::cookie::Key;
use edb::Database;
use http::HeaderValue;

#[derive(Clone)]
pub struct ServerState {
    pub config: AppConfig,
    pub database: Database,
    pub cookie_key: Key,
}

impl FromRef<ServerState> for Key {
    fn from_ref(state: &ServerState) -> Self {
        state.cookie_key.clone()
    }
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub cookie_key: String,
    pub edb_database_url: String,
    pub mock_auth: bool,
    pub web_app_origin: HeaderValue,
}

fn required_env(name: &str) -> String {
    runtime::var(name).unwrap_or_else(|| {
        tracing::error!("{name} not set");
        panic!("{name} required");
    })
}

fn env_or_mock_default(name: &str, default: &str, mock_auth: bool) -> String {
    match runtime::var(name) {
        Some(v) => v,
        None if mock_auth => default.to_string(),
        None => required_env(name),
    }
}

impl AppConfig {
    pub fn new() -> Self {
        let mock_auth = match var("MOCK_AUTH") {
            Ok(v) => {
                let mock_auth = v.parse().unwrap_or(false);
                if cfg!(not(debug_assertions)) && mock_auth {
                    panic!("MOCK_AUTH is not allowed to be set to 'true' outside of a debug build");
                }

                tracing::warn!("MOCK_AUTH explicitly set to {mock_auth}");
                mock_auth
            }
            Err(_e) => false,
        };

        let cookie_key = env_or_mock_default("COOKIE_KEY", &"test-dev-string".repeat(5), mock_auth);

        assert!(
            cookie_key.len() >= 64,
            "COOKIE_KEY must be at least 64 bytes"
        );

        let edb_database_url = required_env("EDB_DATABASE_URL");

        let web_app_origin =
            env_or_mock_default("WEB_APP_ORIGIN", "http://127.0.0.1:13003", mock_auth);
        let web_app_origin = HeaderValue::from_str(&web_app_origin).unwrap_or_else(|e| {
            tracing::error!("WEB_APP_ORIGIN invalid: {e}");
            panic!("WEB_APP_ORIGIN invalid: {e}");
        });

        Self {
            cookie_key,
            edb_database_url,
            mock_auth,
            web_app_origin,
        }
    }
}
