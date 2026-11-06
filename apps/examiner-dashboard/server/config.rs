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
use object_storage::{ObjectStore, S3Config};

#[derive(Clone)]
pub struct ServerState {
    pub config: AppConfig,
    pub database: Database,
    /// Follows no redirects: the OAuth token exchange must not be redirected (SSRF).
    pub http_client: reqwest::Client,
    pub google_client: GoogleClient,
    pub pending_logins: PendingLogins,
    pub cookie_key: Key,
    /// cDb: versioned curriculum content.
    pub curriculum: cdb::Database,
    /// The curriculum bucket: question objects.
    pub curriculum_bucket: ObjectStore,
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
    /// cDb connection string, as eDd's role. Names the database (`curriculum`), never a branch.
    pub cdb_database_url: String,
    /// The curriculum bucket (`CURRICULUM_S3_*`).
    pub curriculum_bucket: S3Config,
    pub web_dir: String,
}

/// `name`'s value. A missing env var and an empty one are treated as being equivalent.
fn env(name: &str) -> Option<String> {
    var(name).ok().filter(|v| !v.is_empty())
}

/// `name`'s value. Panics when missing or empty.
fn required_env(name: &str) -> String {
    env(name).unwrap_or_else(|| {
        tracing::error!("{name} not set");
        panic!("{name} required");
    })
}

/// `name`'s value. When missing or empty: `default` if `mock_auth`, otherwise panics.
fn env_or_mock_default(name: &str, default: &str, mock_auth: bool) -> String {
    match env(name) {
        Some(v) => v,
        None if mock_auth => default.to_string(),
        None => required_env(name),
    }
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

        let google_oauth_id = env_or_mock_default("GOOGLE_OAUTH_ID", "test-dev-string", mock_auth);
        let google_oauth_secret =
            env_or_mock_default("GOOGLE_OAUTH_SECRET", "test-dev-string", mock_auth);
        let google_redirect_url = env_or_mock_default(
            "GOOGLE_REDIRECT_URL",
            "http://127.0.0.1:13002/auth/google/callback",
            mock_auth,
        );
        let cookie_key = env_or_mock_default("COOKIE_KEY", &"test-dev-string".repeat(5), mock_auth);

        assert!(
            cookie_key.len() >= 64,
            "COOKIE_KEY must be at least 64 bytes"
        );

        let edb_database_url = required_env("EDB_DATABASE_URL");
        let cdb_database_url = required_env("CDB_DATABASE_URL");

        let curriculum_bucket = match S3Config::from_env("CURRICULUM") {
            Ok(config) => config,
            Err(e) if mock_auth => {
                tracing::warn!("{e}: curriculum bucket requests will fail");
                mock_s3_config()
            }
            Err(e) => {
                tracing::error!("{e}");
                panic!("{e}");
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
            cdb_database_url,
            curriculum_bucket,
            web_dir,
        }
    }
}

/// Unreachable (port 9, discard): eDd starts, and every bucket request fails.
pub fn mock_s3_config() -> S3Config {
    S3Config {
        endpoint: "http://127.0.0.1:9".to_string(),
        region: "us-east-1".to_string(),
        bucket: "mock".to_string(),
        access_key_id: "test-dev-string".to_string(),
        secret_access_key: "test-dev-string".to_string(),
        force_path_style: true,
    }
}
