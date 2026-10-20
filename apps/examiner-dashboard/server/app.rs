use std::path::PathBuf;

use axum::{Router, routing::get};
use axum_extra::extract::cookie::Key;
use edb::Database;
use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl, basic::BasicClient};
use tower_http::services::{ServeDir, ServeFile};

use crate::config::{AppConfig, PendingLogins, ServerState};
use crate::routes;

pub async fn app(config: &AppConfig) -> Router {
    let google_client = BasicClient::new(ClientId::new(config.google_oauth_id.clone()))
        .set_client_secret(ClientSecret::new(config.google_oauth_secret.clone()))
        .set_auth_uri(
            AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
                .expect("Unreachable. static URL"),
        )
        .set_token_uri(
            TokenUrl::new("https://oauth2.googleapis.com/token".to_string())
                .expect("Unreachable. static URL"),
        )
        .set_redirect_uri(
            RedirectUrl::new(config.google_redirect_url.clone())
                .expect("Unreachable. GOOGLE_REDIRECT_URL validated in config"),
        );

    let http_client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("failed to build HTTP client");

    // Runs pending migrations. Panics so the deploy fails its healthcheck and rolls back.
    let database = Database::connect(&config.edb_database_url)
        .await
        .expect("failed to connect to eDb and migrate");

    let server_state = ServerState {
        config: config.clone(),
        database,
        http_client,
        google_client,
        pending_logins: PendingLogins::default(),
        cookie_key: Key::from(config.cookie_key.as_bytes()),
    };

    let api = Router::new()
        .nest("/auth", routes::auth::router(config.mock_auth))
        .with_state(server_state);

    let web_dir_path = PathBuf::from(&config.web_dir);

    runtime::instrument(api)
        // After instrumentation: probes stay out of request traces and Sentry.
        .route("/healthz", get(routes::get_healthz))
        .nest("/status", routes::status::router())
        .fallback_service(
            ServeDir::new(&web_dir_path)
                .not_found_service(ServeFile::new(web_dir_path.join("404.html"))),
        )
}
