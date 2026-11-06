use axum::{extract::FromRequestParts, http::request::Parts};
use axum_extra::extract::{PrivateCookieJar, cookie::Key};
use edb::User;
use http::StatusCode;
use tracing::warn;

use crate::{
    config::ServerState,
    error::Error,
    routes::auth::{ALLOWED_EMAIL_DOMAIN, SESSION_COOKIE, is_allowed_email},
};

#[derive(Debug, Clone)]
pub struct AuthUser(pub User);

impl FromRequestParts<ServerState> for AuthUser {
    type Rejection = Error;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &ServerState,
    ) -> Result<Self, Self::Rejection> {
        let unauthorized = |msg: &str| Error::Server(StatusCode::UNAUTHORIZED, msg.to_string());

        let jar = PrivateCookieJar::<Key>::from_request_parts(parts, state)
            .await
            .unwrap_or_else(|never| match never {});

        // Missing, or not encrypted with our key.
        let session_id = jar
            .get(SESSION_COOKIE)
            .ok_or_else(|| unauthorized("no session"))?;
        let session = state
            .database
            .find_session(session_id.value())
            .await?
            .ok_or_else(|| unauthorized("unknown or expired session"))?;
        let user = state
            .database
            .find_user_by_id(session.user_id)
            .await?
            .ok_or_else(|| unauthorized("no user for session"))?;

        // Login already enforces the domain. Re-checked so a policy change applies to live sessions.
        if !is_allowed_email(&user.email) {
            warn!(email = user.email, "session user has a disallowed email");
            return Err(Error::Server(
                StatusCode::FORBIDDEN,
                format!("only @{ALLOWED_EMAIL_DOMAIN} addresses may use the examiner dashboard"),
            ));
        }

        Ok(AuthUser(user))
    }
}

#[cfg(test)]
mod tests {
    use axum::{Router, body::Body, routing::get};
    use axum_extra::extract::cookie::Cookie;
    use http::{Request, header::COOKIE};
    use tower::ServiceExt;

    use super::*;
    use object_storage::ObjectStore;

    use crate::config::{AppConfig, PendingLogins, mock_s3_config};
    use crate::routes::auth::google::google_client;

    /// Never connects: these requests are rejected before any query.
    fn server_state() -> ServerState {
        let config = AppConfig {
            cookie_key: "k".repeat(64),
            edb_database_url: "postgres://unused@127.0.0.1:1/unused".to_string(),
            google_oauth_id: "id".to_string(),
            google_oauth_secret: "secret".to_string(),
            google_redirect_url: "http://127.0.0.1:13002/auth/google/callback".to_string(),
            mock_auth: false,
            cdb_database_url: "mysql://unused@127.0.0.1:1/unused".to_string(),
            curriculum_storage: mock_s3_config(),
            web_dir: "dist".to_string(),
        };
        let pool = sqlx::PgPool::connect_lazy(&config.edb_database_url).expect("valid URL");

        ServerState {
            database: edb::Database::from_pool(pool),
            http_client: reqwest::Client::new(),
            google_client: google_client(&config),
            pending_logins: PendingLogins::default(),
            cookie_key: Key::from(config.cookie_key.as_bytes()),
            curriculum: cdb::Database::connect_lazy(&config.cdb_database_url).expect("valid URL"),
            curriculum_store: ObjectStore::new(&config.curriculum_storage),
            config,
        }
    }

    async fn status(request: Request<Body>) -> StatusCode {
        let app = Router::new()
            .route("/", get(|_: AuthUser| async { "ok" }))
            .with_state(server_state());

        app.oneshot(request).await.expect("infallible").status()
    }

    #[tokio::test]
    async fn rejects_missing_session_cookie() {
        let request = Request::get("/")
            .body(Body::empty())
            .expect("valid request");

        assert_eq!(status(request).await, StatusCode::UNAUTHORIZED);
    }

    /// A plaintext or forged sid fails decryption, so it reads as no session.
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
