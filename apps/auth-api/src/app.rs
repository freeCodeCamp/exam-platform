use axum::{Router, routing::get};
use edb::Database;

use crate::routes;

pub async fn app(edb_database_url: String) -> Router {
    // As role `aapi`: the deploy ran edb-migrate first. Panics so the deploy fails its healthcheck
    // and rolls back.
    let database = Database::connect(&edb_database_url)
        .await
        .expect("failed to connect to eDb");

    let api = Router::new().with_state(database);

    runtime::instrument(api)
        // After instrumentation: probes stay out of request traces and Sentry.
        .route("/healthz", get(routes::get_healthz))
}
