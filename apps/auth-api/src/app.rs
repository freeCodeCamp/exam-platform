use axum::{Router, routing::get};
use axum_extra::extract::cookie::Key;
use edb::Database;
use http::{Method, header::CONTENT_TYPE};
use tower_http::cors::CorsLayer;

use crate::config::{AppConfig, ServerState};
use crate::routes;

pub async fn app(config: &AppConfig) -> Router {
    let database = Database::connect(&config.edb_database_url)
        .await
        .expect("failed to connect to eDb");

    router(ServerState {
        config: config.clone(),
        database,
        cookie_key: Key::from(config.cookie_key.as_bytes()),
    })
}

pub fn router(server_state: ServerState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(server_state.config.web_app_origins.clone())
        .allow_methods([Method::GET, Method::POST, Method::DELETE])
        .allow_headers([CONTENT_TYPE])
        .allow_credentials(true);

    let api = Router::new()
        .nest("/auth", routes::auth::router(server_state.config.mock_auth))
        .layer(cors)
        .with_state(server_state);

    runtime::instrument(api)
        // After instrumentation: probes stay out of request traces and Sentry.
        .route("/healthz", get(routes::get_healthz))
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use http::{
        HeaderValue, Request, StatusCode,
        header::{
            ACCESS_CONTROL_ALLOW_CREDENTIALS, ACCESS_CONTROL_ALLOW_ORIGIN,
            ACCESS_CONTROL_REQUEST_METHOD, ORIGIN,
        },
    };
    use tower::ServiceExt;

    use super::*;
    use crate::testing::{WEB_APP_ORIGINS, server_state};

    fn preflight(origin: &str) -> Request<Body> {
        Request::options("/auth/session")
            .header(ORIGIN, origin)
            .header(ACCESS_CONTROL_REQUEST_METHOD, "GET")
            .body(Body::empty())
            .expect("valid request")
    }

    fn dev_login(email: &str) -> Request<Body> {
        Request::post("/auth/dev/login")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(format!(
                r#"{{"name":"Camper","email":"{email}"}}"#
            )))
            .expect("valid request")
    }

    #[tokio::test]
    async fn allows_credentialed_requests_from_every_web_app_origin() {
        for origin in WEB_APP_ORIGINS {
            let response = router(server_state(false))
                .oneshot(preflight(origin))
                .await
                .expect("infallible");

            let headers = response.headers();
            assert_eq!(
                headers.get(ACCESS_CONTROL_ALLOW_ORIGIN),
                Some(&HeaderValue::from_static(origin))
            );
            assert_eq!(
                headers.get(ACCESS_CONTROL_ALLOW_CREDENTIALS),
                Some(&HeaderValue::from_static("true"))
            );
        }
    }

    #[tokio::test]
    async fn rejects_other_origins() {
        let response = router(server_state(false))
            .oneshot(preflight("https://evil.example"))
            .await
            .expect("infallible");

        assert!(
            response
                .headers()
                .get(ACCESS_CONTROL_ALLOW_ORIGIN)
                .is_none()
        );
    }

    #[tokio::test]
    async fn dev_login_needs_mock_auth() {
        let response = router(server_state(false))
            .oneshot(dev_login("camper@example.com"))
            .await
            .expect("infallible");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn dev_login_rejects_invalid_email() {
        let response = router(server_state(true))
            .oneshot(dev_login("camper"))
            .await
            .expect("infallible");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
