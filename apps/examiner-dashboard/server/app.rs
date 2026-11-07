use std::path::PathBuf;

use axum::{Router, routing::get};
use axum_extra::extract::cookie::Key;
use edb::Database;
use tower_http::services::{ServeDir, ServeFile};

use crate::config::{AppConfig, PendingLogins, ServerState};
use crate::routes::{self, auth::google::google_client};
use object_storage::ObjectStore;

pub async fn app(config: &AppConfig) -> Router {
    let google_client = google_client(config);

    let http_client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("failed to build HTTP client");

    let database = Database::connect(&config.edb_database_url)
        .await
        .expect("failed to connect to eDb");

    let server_state = ServerState {
        config: config.clone(),
        database,
        http_client,
        google_client,
        pending_logins: PendingLogins::default(),
        cookie_key: Key::from(config.cookie_key.as_bytes()),
        curriculum_bucket: ObjectStore::new(&config.curriculum_bucket),
    };

    let api = Router::new()
        .nest("/auth", routes::auth::router(config.mock_auth))
        .nest("/api/objects", routes::objects::router())
        .nest("/api/object-store", routes::object_store::router())
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
