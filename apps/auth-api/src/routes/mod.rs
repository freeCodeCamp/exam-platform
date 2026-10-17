use axum::http::header::{CACHE_CONTROL, HeaderValue};
use axum::response::IntoResponse;

pub async fn get_healthz() -> impl IntoResponse {
    (
        [(CACHE_CONTROL, HeaderValue::from_static("no-store"))],
        "ok",
    )
}

pub async fn get_status_ping() -> Response {
    let mut response = Response::new("pong".into());
    response.headers_mut().insert(
        CACHE_CONTROL,
        "no-cache"
            .parse()
            .expect("Unreachable. static str into HeaderValue"),
    );
    response.headers_mut().insert(
        CONTENT_TYPE,
        "text/plain; charset=utf-8"
            .parse()
            .expect("Unreachable. static str into HeaderValue"),
    );
    response
}
