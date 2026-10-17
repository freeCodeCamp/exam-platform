use std::time::{Duration, SystemTime};

use axum::{
    Json, Router,
    extract::State,
    response::{IntoResponse, Redirect},
    routing::{delete, get, post},
};
use axum_extra::extract::{
    PrivateCookieJar,
    cookie::{Cookie, SameSite},
};
use http::StatusCode;
use oauth2::CsrfToken;
use serde::Deserialize;
use tracing::{instrument, warn};

use crate::{
    config::ServerState,
    database::{NewUser, Session, User},
    error::Error,
};

pub mod google;

pub const ALLOWED_EMAIL_DOMAIN: &str = "freecodecamp.org";
const SESSION_TTL: Duration = Duration::from_secs(60 * 60);
const SESSION_COOKIE: &str = "sid";

pub fn router(mock_auth: bool) -> Router<ServerState> {
    let router = Router::new()
        .route("/google/login", get(google::get_login))
        .route("/google/callback", get(google::get_callback))
        .route("/logout", delete(delete_logout));

    if mock_auth {
        router.route("/dev/login", post(post_dev_login))
    } else {
        router
    }
}

/// `true` for `<local>@freecodecamp.org`, case-insensitive on the domain.
pub fn is_allowed_email(email: &str) -> bool {
    email.rsplit_once('@').is_some_and(|(local, domain)| {
        !local.is_empty() && domain.eq_ignore_ascii_case(ALLOWED_EMAIL_DOMAIN)
    })
}

/// Stores a new session for `user` and sets its sid cookie.
async fn create_session(
    server_state: &ServerState,
    jar: PrivateCookieJar,
    user: &User,
    expires_in: Duration,
) -> Result<PrivateCookieJar, Error> {
    let session = Session {
        // 32 bytes from a CSPRNG, base64url.
        session_id: CsrfToken::new_random_len(32).into_secret(),
        user_id: user.id.clone(),
        expires_at: SystemTime::now() + expires_in,
    };

    server_state.database.insert_session(&session).await?;

    let cookie = Cookie::build((SESSION_COOKIE, session.session_id))
        .path("/")
        .secure(true)
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(expires_in.try_into()?);

    Ok(jar.add(cookie))
}

/// Logs the user out by deleting the db session(s), and unsetting the sid
#[instrument(skip_all, err(Debug), level = "debug")]
pub async fn delete_logout(
    jar: PrivateCookieJar,
    State(server_state): State<ServerState>,
) -> Result<PrivateCookieJar, Error> {
    let session_id = jar
        .get(SESSION_COOKIE)
        .map(|cookie| cookie.value().to_owned())
        .ok_or(Error::Server(
            StatusCode::UNAUTHORIZED,
            "invalid sid in cookie jar".to_string(),
        ))?;

    if let Some(session) = server_state.database.find_session(&session_id).await? {
        server_state
            .database
            .delete_user_sessions(&session.user_id)
            .await?;
    }

    Ok(jar.remove(Cookie::build(SESSION_COOKIE).path("/")))
}

#[derive(Deserialize)]
pub struct DevLoginBody {
    pub name: String,
    pub email: String,
}

/// Dev login route, only mounted when `MOCK_AUTH=true`
///
/// Takes a name and email as body parameters, creates a user if one does not exist,
/// and creates a session for that user, setting the sid cookie in the response.
#[instrument(skip_all, err(Debug), level = "debug")]
pub async fn post_dev_login(
    jar: PrivateCookieJar,
    State(server_state): State<ServerState>,
    Json(user_body): Json<DevLoginBody>,
) -> Result<impl IntoResponse, Error> {
    if !is_allowed_email(&user_body.email) {
        warn!(
            email = user_body.email,
            "dev login attempt from disallowed domain"
        );
        return Err(Error::Server(
            StatusCode::FORBIDDEN,
            format!("only @{ALLOWED_EMAIL_DOMAIN} addresses may log in"),
        ));
    }

    let dev_user = server_state
        .database
        .find_user_by_email(&user_body.email)
        .await?;

    let user = match dev_user {
        Some(user) => user,
        None => {
            server_state
                .database
                .insert_user(NewUser {
                    name: user_body.name,
                    email: user_body.email,
                    google_id: None,
                    picture: None,
                })
                .await?
        }
    };

    let jar = create_session(&server_state, jar, &user, SESSION_TTL).await?;

    Ok((jar, Redirect::to("/")))
}
