use axum_extra::extract::cookie::Key;
use http::HeaderValue;

use crate::config::{AppConfig, ServerState};

pub const WEB_APP_ORIGIN: &str = "http://127.0.0.1:13003";

pub fn server_state(mock_auth: bool) -> ServerState {
    let config = AppConfig {
        cookie_key: "k".repeat(64),
        edb_database_url: "postgres://unused@127.0.0.1:1/unused".to_string(),
        mock_auth,
        web_app_origin: HeaderValue::from_static(WEB_APP_ORIGIN),
    };
    let pool = sqlx::PgPool::connect_lazy(&config.edb_database_url).expect("valid URL");

    ServerState {
        database: edb::Database::from_pool(pool),
        cookie_key: Key::from(config.cookie_key.as_bytes()),
        config,
    }
}
