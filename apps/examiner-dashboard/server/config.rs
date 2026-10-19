use std::{
    collections::HashMap,
    env::var,
    sync::{Arc, Mutex, PoisonError},
    time::{Duration, Instant},
};

use axum::extract::FromRef;
use axum_extra::extract::cookie::Key;
use oauth2::PkceCodeVerifier;

use crate::database::Database;
use crate::routes::auth::google::GoogleClient;

#[derive(Clone)]
pub struct ServerState {
    pub config: AppConfig,
    pub database: Database,
    /// Follows no redirects: the OAuth token exchange must not be redirected (SSRF).
    pub http_client: reqwest::Client,
    pub google_client: GoogleClient,
    pub pending_logins: PendingLogins,
    pub cookie_key: Key,
}

impl FromRef<ServerState> for Key {
    fn from_ref(state: &ServerState) -> Self {
        state.cookie_key.clone()
    }
}

/// How long a user has to complete the Google consent screen.
pub const PENDING_LOGIN_TTL: Duration = Duration::from_secs(10 * 60);

/// PKCE verifiers of Google logins in flight, keyed by CSRF state.
///
/// TODO: In memory. Consider moving to DB.
#[derive(Clone, Default)]
pub struct PendingLogins(Arc<Mutex<HashMap<String, PendingLogin>>>);

struct PendingLogin {
    pkce_code_verifier: PkceCodeVerifier,
    created_at: Instant,
}

impl PendingLogins {
    pub fn insert(&self, csrf_state: String, pkce_code_verifier: PkceCodeVerifier) {
        let mut pending = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        // Abandoned logins are never taken, so prune on insert.
        pending.retain(|_, login| login.created_at.elapsed() < PENDING_LOGIN_TTL);
        pending.insert(
            csrf_state,
            PendingLogin {
                pkce_code_verifier,
                created_at: Instant::now(),
            },
        );
    }

    /// Removes the login, so each CSRF state is usable once.
    pub fn take(&self, csrf_state: &str) -> Option<PkceCodeVerifier> {
        let mut pending = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        pending
            .remove(csrf_state)
            .filter(|login| login.created_at.elapsed() < PENDING_LOGIN_TTL)
            .map(|login| login.pkce_code_verifier)
    }
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Encrypts private cookies. At least 64 bytes. Rotating logs everyone out.
    pub cookie_key: String,
    pub google_oauth_id: String,
    pub google_oauth_secret: String,
    pub google_redirect_url: String,
    pub mock_auth: bool,
    pub web_dir: String,
}

impl AppConfig {
    pub fn new() -> Self {
        let mock_auth = match var("MOCK_AUTH") {
            Ok(v) => {
                let mock_auth = v.parse().unwrap_or(false);
                if cfg!(not(debug_assertions)) && mock_auth {
                    panic!("MOCK_AUTH is not allowed to be set to 'true' outside of a debug build");
                }

                tracing::warn!("MOCK_AUTH explicitly set to {mock_auth}");
                mock_auth
            }
            Err(_e) => false,
        };

        let google_oauth_id = match var("GOOGLE_OAUTH_ID") {
            Ok(v) => v,
            Err(_e) => {
                // If no value provided, but MOCK_AUTH=true, then default, otherwise panic
                if !mock_auth {
                    tracing::error!("GOOGLE_OAUTH_ID not set");
                    panic!("GOOGLE_OAUTH_ID required");
                }
                "test-dev-string".to_string()
            }
        };
        assert!(
            !google_oauth_id.is_empty(),
            "GOOGLE_OAUTH_ID must not be empty"
        );

        let google_oauth_secret = match var("GOOGLE_OAUTH_SECRET") {
            Ok(v) => v,
            Err(_e) => {
                // If no value provided, but MOCK_AUTH=true, then default, otherwise panic
                if !mock_auth {
                    tracing::error!("GOOGLE_OAUTH_SECRET not set");
                    panic!("GOOGLE_OAUTH_SECRET required");
                }
                "test-dev-string".to_string()
            }
        };
        assert!(
            !google_oauth_secret.is_empty(),
            "GOOGLE_OAUTH_SECRET must not be empty"
        );

        let google_redirect_url = match var("GOOGLE_REDIRECT_URL") {
            Ok(v) => v,
            Err(_e) => {
                // If no value provided, but MOCK_AUTH=true, then default, otherwise panic
                if !mock_auth {
                    tracing::error!("GOOGLE_REDIRECT_URL not set");
                    panic!("GOOGLE_REDIRECT_URL required");
                }
                "http://127.0.0.1:13002/auth/google/callback".to_string()
            }
        };
        assert!(
            url::Url::parse(&google_redirect_url).is_ok(),
            "GOOGLE_REDIRECT_URL must be a valid URL"
        );

        let cookie_key = match var("COOKIE_KEY") {
            Ok(v) => v,
            Err(_e) => {
                // If no value provided, but MOCK_AUTH=true, then default, otherwise panic
                if !mock_auth {
                    tracing::error!("COOKIE_KEY not set");
                    panic!("COOKIE_KEY required");
                }
                "test-dev-string".repeat(5)
            }
        };
        assert!(
            cookie_key.len() >= 64,
            "COOKIE_KEY must be at least 64 bytes"
        );

        let web_dir = match var("WEB_DIR") {
            Ok(v) => {
                tracing::info!("WEB_DIR explicitly set to {v}");
                v
            }
            Err(_e) => "dist".to_string(),
        };

        Self {
            cookie_key,
            google_oauth_id,
            google_oauth_secret,
            google_redirect_url,
            mock_auth,
            web_dir,
        }
    }
}
