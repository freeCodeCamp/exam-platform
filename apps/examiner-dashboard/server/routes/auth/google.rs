use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect},
};
use axum_extra::extract::{
    PrivateCookieJar,
    cookie::{Cookie, SameSite},
};
use http::StatusCode;
use oauth2::{
    AccessToken, AuthorizationCode, CsrfToken, EmptyExtraTokenFields, EndpointNotSet, EndpointSet,
    HttpRequest, HttpResponse, PkceCodeChallenge, PkceCodeVerifier, Scope, StandardTokenResponse,
    TokenResponse,
    basic::{BasicClient, BasicTokenType},
};
use reqwest::Client;
use runtime::sentry;
use tracing::{info, warn};
use url::Url;

use super::{ALLOWED_EMAIL_DOMAIN, SESSION_TTL, create_session, is_allowed_email};
use crate::{
    config::{PENDING_LOGIN_TTL, ServerState},
    database::NewUser,
    error::Error,
};

pub type GoogleClient =
    BasicClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet>;

/// Ties the callback to the browser that started the login (login CSRF).
const STATE_COOKIE: &str = "google_oauth_state";
const STATE_COOKIE_PATH: &str = "/auth/google";

pub async fn get_login(
    jar: PrivateCookieJar,
    State(server_state): State<ServerState>,
) -> impl IntoResponse {
    if server_state.config.mock_auth {
        let redirect_url = Url::parse_with_params(
            &server_state.config.google_redirect_url,
            &[("code", "anything"), ("state", "anything")],
        )
        .expect("Unreachable. GOOGLE_REDIRECT_URL validated in config");
        return (jar, Redirect::to(redirect_url.as_str()));
    }

    let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();

    let (authorize_url, csrf_state) = server_state
        .google_client
        .authorize_url(CsrfToken::new_random)
        .add_scopes([
            Scope::new("openid".to_string()),
            Scope::new("email".to_string()),
            Scope::new("profile".to_string()),
        ])
        // Hint only: narrows Google's account chooser. The callback enforces the domain.
        .add_extra_param("hd", ALLOWED_EMAIL_DOMAIN)
        .set_pkce_challenge(pkce_code_challenge)
        .url();

    let cookie = Cookie::build((STATE_COOKIE, csrf_state.secret().clone()))
        .path(STATE_COOKIE_PATH)
        .secure(true)
        .http_only(true)
        // Lax: sent on the top-level redirect back from Google.
        .same_site(SameSite::Lax)
        .max_age(
            PENDING_LOGIN_TTL
                .try_into()
                .expect("Unreachable. static duration"),
        );

    server_state
        .pending_logins
        .insert(csrf_state.into_secret(), pkce_code_verifier);

    (jar.add(cookie), Redirect::to(authorize_url.as_str()))
}

#[derive(Debug, serde::Deserialize)]
pub struct AuthCallbackQueryParams {
    code: AuthorizationCode,
    state: String,
}

/// Google's OpenID Connect userinfo claims.
#[derive(Debug, serde::Deserialize)]
pub struct GoogleUserInfo {
    sub: String,
    email: Option<String>,
    #[serde(default)]
    email_verified: bool,
    /// Google Workspace domain. Absent for consumer accounts.
    hd: Option<String>,
    name: Option<String>,
    picture: Option<String>,
}

pub async fn get_callback(
    jar: PrivateCookieJar,
    State(server_state): State<ServerState>,
    Query(params): Query<AuthCallbackQueryParams>,
) -> Result<impl IntoResponse, Error> {
    let AuthCallbackQueryParams { code, state } = params;
    let mock_auth = server_state.config.mock_auth;

    let pkce_code_verifier = if mock_auth {
        PkceCodeVerifier::new("mock".to_string())
    } else {
        // State must match this browser's cookie, and a login in flight on this server.
        let cookie_state = jar
            .get(STATE_COOKIE)
            .map(|cookie| cookie.value().to_owned());
        if cookie_state.as_deref() != Some(state.as_str()) {
            warn!("google callback state does not match cookie");
            return Err(Error::Server(
                StatusCode::UNAUTHORIZED,
                "invalid oauth state".to_string(),
            ));
        }
        server_state
            .pending_logins
            .take(&state)
            .ok_or(Error::Server(
                StatusCode::UNAUTHORIZED,
                "unknown or expired oauth state".to_string(),
            ))?
    };
    let jar = jar.remove(Cookie::build(STATE_COOKIE).path(STATE_COOKIE_PATH));

    let token = get_access_token(
        code,
        pkce_code_verifier,
        &server_state.google_client,
        &server_state.http_client,
        mock_auth,
    )
    .await?;

    let google_user_info = get_google_user_info(
        token.access_token().secret(),
        &server_state.http_client,
        mock_auth,
    )
    .await?;

    let email = match google_user_info.email {
        Some(email) if google_user_info.email_verified => email,
        _ => {
            return Err(Error::Server(
                StatusCode::UNAUTHORIZED,
                "no verified email associated with Google account".to_string(),
            ));
        }
    };

    // `hd` proves the account is managed by the freeCodeCamp Workspace, not just a matching address.
    let is_workspace_account = google_user_info
        .hd
        .as_deref()
        .is_some_and(|hd| hd.eq_ignore_ascii_case(ALLOWED_EMAIL_DOMAIN));
    if !is_workspace_account || !is_allowed_email(&email) {
        warn!({ email }, "login attempt from disallowed domain");
        sentry::metrics::counter("auth.login", 1)
            .attribute("outcome", "forbidden")
            .capture();
        return Err(Error::Server(
            StatusCode::FORBIDDEN,
            format!("only @{ALLOWED_EMAIL_DOMAIN} addresses may log in"),
        ));
    }

    let user = match server_state.database.find_user_by_email(&email).await? {
        Some(user) => {
            server_state
                .database
                .update_user_google_profile(
                    &user.id,
                    &google_user_info.sub,
                    google_user_info.picture.as_deref(),
                )
                .await?;
            user
        }
        None => {
            info!({ email }, "creating user on first login");
            server_state
                .database
                .insert_user(NewUser {
                    name: google_user_info.name.unwrap_or_else(|| email.clone()),
                    email,
                    google_id: Some(google_user_info.sub),
                    picture: google_user_info.picture,
                })
                .await?
        }
    };

    let expires_in = token.expires_in().unwrap_or(SESSION_TTL);
    let jar = create_session(&server_state, jar, &user, expires_in).await?;

    sentry::metrics::counter("auth.login", 1)
        .attribute("outcome", "success")
        .capture();

    Ok((jar, Redirect::to("/")))
}

async fn get_google_user_info(
    access_token: &str,
    http_client: &Client,
    mock_auth: bool,
) -> Result<GoogleUserInfo, Error> {
    if mock_auth {
        let google_user_info = GoogleUserInfo {
            sub: "0".to_string(),
            email: Some("camperbot@freecodecamp.org".to_string()),
            email_verified: true,
            hd: Some(ALLOWED_EMAIL_DOMAIN.to_string()),
            name: Some("Camperbot".to_string()),
            picture: None,
        };

        return Ok(google_user_info);
    }

    let res = http_client
        .get("https://openidconnect.googleapis.com/v1/userinfo")
        .bearer_auth(access_token)
        .send()
        .await?
        .error_for_status()?;
    let google_user_info: GoogleUserInfo = res.json().await?;

    Ok(google_user_info)
}

async fn get_access_token(
    code: AuthorizationCode,
    pkce_code_verifier: PkceCodeVerifier,
    google_client: &GoogleClient,
    http_client: &Client,
    mock_auth: bool,
) -> Result<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>, Error> {
    if mock_auth {
        let token = StandardTokenResponse::new(
            AccessToken::new("camperbot-access-token".to_string()),
            BasicTokenType::Bearer,
            EmptyExtraTokenFields {},
        );
        return Ok(token);
    }

    // Request access token from Google
    let token = google_client
        .exchange_code(code)
        .set_pkce_verifier(pkce_code_verifier)
        .request_async(&|request| send_oauth_request(http_client.clone(), request))
        .await
        .map_err(|e| Error::Server(StatusCode::INTERNAL_SERVER_ERROR, format!("{e}")))?;

    Ok(token)
}

/// oauth2's bundled client is reqwest 0.12, so its requests go through the workspace reqwest.
/// Owns `http_client`: a borrowed one makes the callback future not `Send`.
async fn send_oauth_request(
    http_client: Client,
    request: HttpRequest,
) -> Result<HttpResponse, reqwest::Error> {
    let response = http_client.execute(request.try_into()?).await?;

    let mut http_response = HttpResponse::new(Vec::new());
    *http_response.status_mut() = response.status();
    *http_response.headers_mut() = response.headers().clone();
    *http_response.body_mut() = response.bytes().await?.to_vec();

    Ok(http_response)
}
