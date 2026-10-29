use axum::{Router, routing::get};
use edb::Database;

use crate::config::AppConfig;
use crate::routes;

pub async fn app(config: &AppConfig) -> Router {
    let database = Database::connect(&config.edb_database_url)
        .await
        .expect("failed to connect to eDb");

    let api = Router::new().with_state(database);

    runtime::instrument(api)
        // After instrumentation: probes stay out of request traces and Sentry.
        .route("/healthz", get(routes::get_healthz))
}
