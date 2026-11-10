use std::time::Duration;

use axum::{
    Json, Router,
    extract::State,
    http::header::{CACHE_CONTROL, HeaderValue},
    response::IntoResponse,
    routing::{delete, get, post},
};
use axum_extra::extract::{
    PrivateCookieJar,
    cookie::{Cookie, SameSite},
};
use edb::{Candidate, CandidateSession, NewCandidate};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tracing::instrument;

use crate::{config::ServerState, error::Error, extractor::AuthCandidate};

pub const SESSION_COOKIE: &str = "candidate_sid";
const SESSION_TTL: Duration = Duration::from_secs(60 * 60);

pub fn router(mock_auth: bool) -> Router<ServerState> {
    let router = Router::new()
        .route("/session", get(get_session))
        .route("/logout", delete(delete_logout));

    if mock_auth {
        router.route("/dev/login", post(post_dev_login))
    } else {
        router
    }
}

fn new_session_id() -> String {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).expect("OS random source unavailable");
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn is_email(email: &str) -> bool {
    email
        .split_once('@')
        .is_some_and(|(local, domain)| !local.is_empty() && !domain.is_empty())
}

async fn create_session(
    server_state: &ServerState,
    jar: PrivateCookieJar,
    candidate: &Candidate,
    expires_in: Duration,
) -> Result<PrivateCookieJar, Error> {
    let session = CandidateSession {
        session_id: new_session_id(),
        candidate_id: candidate.id,
        expires_at: OffsetDateTime::now_utc() + expires_in,
    };

    server_state
        .database
        .insert_candidate_session(&session)
        .await?;

    let cookie = Cookie::build((SESSION_COOKIE, session.session_id))
        .path("/")
        .secure(true)
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(expires_in.try_into()?);

    Ok(jar.add(cookie))
}

#[derive(Debug, Serialize)]
pub struct SessionCandidate {
    pub id: String,
    pub name: String,
    pub email: String,
}

impl From<Candidate> for SessionCandidate {
    fn from(candidate: Candidate) -> Self {
        Self {
            id: candidate.id.to_string(),
            name: candidate.name,
            email: candidate.email,
        }
    }
}

#[instrument(skip_all, level = "debug")]
pub async fn get_session(AuthCandidate(candidate): AuthCandidate) -> impl IntoResponse {
    (
        [(CACHE_CONTROL, HeaderValue::from_static("no-store"))],
        Json(SessionCandidate::from(candidate)),
    )
}

#[instrument(skip_all, err(Debug), level = "debug")]
pub async fn delete_logout(
    AuthCandidate(candidate): AuthCandidate,
    jar: PrivateCookieJar,
    State(server_state): State<ServerState>,
) -> Result<(PrivateCookieJar, StatusCode), Error> {
    server_state
        .database
        .delete_candidate_sessions(candidate.id)
        .await?;

    Ok((
        jar.remove(Cookie::build(SESSION_COOKIE).path("/")),
        StatusCode::NO_CONTENT,
    ))
}

#[derive(Deserialize)]
pub struct DevLoginBody {
    pub name: String,
    pub email: String,
}

#[instrument(skip_all, err(Debug), level = "debug")]
pub async fn post_dev_login(
    jar: PrivateCookieJar,
    State(server_state): State<ServerState>,
    Json(body): Json<DevLoginBody>,
) -> Result<impl IntoResponse, Error> {
    if !is_email(&body.email) {
        return Err(Error::bad_request("invalid email"));
    }

    let candidate = match server_state
        .database
        .find_candidate_by_email(&body.email)
        .await?
    {
        Some(candidate) => candidate,
        None => {
            server_state
                .database
                .insert_candidate(NewCandidate {
                    name: body.name,
                    email: body.email,
                })
                .await?
        }
    };

    let jar = create_session(&server_state, jar, &candidate, SESSION_TTL).await?;

    Ok((jar, Json(SessionCandidate::from(candidate))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_ids_are_random_hex() {
        let a = new_session_id();
        let b = new_session_id();

        assert_eq!(a.len(), 64);
        assert!(a.bytes().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(a, b);
    }

    #[test]
    fn validates_email_shape() {
        assert!(is_email("camper@example.com"));
        assert!(!is_email("camper"));
        assert!(!is_email("@example.com"));
        assert!(!is_email("camper@"));
    }
}
