use std::{
    collections::HashMap,
    env::var,
    sync::{Arc, Mutex, PoisonError},
    time::{Duration, Instant},
};

use axum::extract::FromRef;
use axum_extra::extract::cookie::Key;
use oauth2::PkceCodeVerifier;

use edb::Database;

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
    /// eDb connection string. Hosted Postgres: add `?sslmode=verify-full`.
    pub edb_database_url: String,
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

        let get_env_with_default = |name: &str, default: &str| {
            // A missing env var and an empty one are treated as being equivalent
            let val = var(name).unwrap_or_default();

            if !val.is_empty() {
                return val;
            }

            if !mock_auth {
                tracing::error!("{name} not set");
                panic!("{name} required");
            }

            default.to_string()
        };

        let google_oauth_id = get_env_with_default("GOOGLE_OAUTH_ID", "test-dev-string");
        let google_oauth_secret = get_env_with_default("GOOGLE_OAUTH_SECRET", "test-dev-string");
        let google_redirect_url = get_env_with_default(
            "GOOGLE_REDIRECT_URL",
            "http://127.0.0.1:13002/auth/google/callback",
        );
        let cookie_key = get_env_with_default("COOKIE_KEY", &"test-dev-string".repeat(5));

        assert!(
            cookie_key.len() >= 64,
            "COOKIE_KEY must be at least 64 bytes"
        );

        let edb_database_url = match var("EDB_DATABASE_URL") {
            Ok(v) => v,
            Err(_e) => {
                tracing::error!("EDB_DATABASE_URL not set");
                panic!("EDB_DATABASE_URL required");
            }
        };

        let web_dir = match var("WEB_DIR") {
            Ok(v) => {
                tracing::info!("WEB_DIR explicitly set to {v}");
                v
            }
            Err(_e) => "dist".to_string(),
        };

        Self {
            cookie_key,
            edb_database_url,
            google_oauth_id,
            google_oauth_secret,
            google_redirect_url,
            mock_auth,
            web_dir,
        }
    }
}
