use axum::http::header::{CACHE_CONTROL, HeaderValue};
use axum::response::IntoResponse;

pub mod auth;
pub mod status;

/// Liveness probe for Docker and the platform runner.
pub async fn get_healthz() -> impl IntoResponse {
    (
        [(CACHE_CONTROL, HeaderValue::from_static("no-store"))],
        "ok",
    )
}
