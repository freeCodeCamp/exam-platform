use axum::{Router, routing::get};
use mdb::Database;

use crate::routes;

pub async fn app(mdb_database_url: String) -> Router {
    // Runs pending mDb migrations: aAPI owns the schema. Panics so the deploy fails its
    // healthcheck and rolls back.
    let database = Database::connect(&mdb_database_url)
        .await
        .expect("failed to connect to mDb and migrate");

    let api = Router::new().with_state(database);

    runtime::instrument(api)
        // After instrumentation: probes stay out of request traces and Sentry.
        .route("/healthz", get(routes::get_healthz))
}
