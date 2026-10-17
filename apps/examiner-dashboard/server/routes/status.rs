use axum::Router;
use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE};
use axum::response::Response;
use axum::routing::get;

pub async fn get_ping() -> Response {
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

pub fn router() -> Router {
    Router::new().route("/ping", get(get_ping))
}
