use axum::{extract::FromRequestParts, http::request::Parts};
use axum_extra::extract::{PrivateCookieJar, cookie::Key};
use edb::Candidate;

use crate::{config::ServerState, error::Error, routes::auth::SESSION_COOKIE};

#[derive(Debug, Clone)]
pub struct AuthCandidate(pub Candidate);

impl FromRequestParts<ServerState> for AuthCandidate {
    type Rejection = Error;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &ServerState,
    ) -> Result<Self, Self::Rejection> {
        let jar = PrivateCookieJar::<Key>::from_request_parts(parts, state)
            .await
            .unwrap_or_else(|never| match never {});

        let session_id = jar
            .get(SESSION_COOKIE)
            .ok_or_else(|| Error::unauthorized("no session"))?;
        let session = state
            .database
            .find_candidate_session(session_id.value())
            .await?
            .ok_or_else(|| Error::unauthorized("unknown or expired session"))?;
        let candidate = state
            .database
            .find_candidate_by_id(session.candidate_id)
            .await?
            .ok_or_else(|| Error::unauthorized("no candidate for session"))?;

        Ok(AuthCandidate(candidate))
    }
}

#[cfg(test)]
mod tests {
    use axum::{Router, body::Body, routing::get};
    use axum_extra::extract::cookie::Cookie;
    use http::{Request, StatusCode, header::COOKIE};
    use tower::ServiceExt;

    use super::*;
    use crate::testing::server_state;

    async fn status(request: Request<Body>) -> StatusCode {
        let app = Router::new()
            .route("/", get(|_: AuthCandidate| async { "ok" }))
            .with_state(server_state(false));

        app.oneshot(request).await.expect("infallible").status()
    }

    #[tokio::test]
    async fn rejects_missing_session_cookie() {
        let request = Request::get("/")
            .body(Body::empty())
            .expect("valid request");

        assert_eq!(status(request).await, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn rejects_unencrypted_session_cookie() {
        let cookie = Cookie::new(SESSION_COOKIE, "forged");
        let request = Request::get("/")
            .header(COOKIE, cookie.to_string())
            .body(Body::empty())
            .expect("valid request");

        assert_eq!(status(request).await, StatusCode::UNAUTHORIZED);
    }
}
