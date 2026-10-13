use std::{ffi::OsString, path::PathBuf};

use axum::{Router, routing::get};
use runtime::ConfigError;
use tower_http::services::{ServeDir, ServeFile};

use crate::routes;

/// `WEB_DIR`: the built client, `client/dist`. Required in release builds, so an image without the
/// client fails to start instead of passing its health check while `/` answers 404. Optional in
/// debug builds, where Astro's dev server serves the client.
pub fn web_dir() -> Result<Option<PathBuf>, ConfigError> {
    parse_web_dir(std::env::var_os("WEB_DIR"))
}

fn parse_web_dir(raw: Option<OsString>) -> Result<Option<PathBuf>, ConfigError> {
    let Some(dir) = raw.filter(|v| !v.is_empty()).map(PathBuf::from) else {
        return if cfg!(debug_assertions) {
            Ok(None)
        } else {
            Err(ConfigError::new(
                "WEB_DIR must be set in release builds: the built client's directory",
            ))
        };
    };
    if !dir.join("index.html").is_file() {
        return Err(ConfigError::new(format!(
            "WEB_DIR {} has no index.html: build the client",
            dir.display()
        )));
    }
    Ok(Some(dir))
}

pub async fn app(web_dir: Option<PathBuf>) -> Router {
    let router = runtime::instrument(Router::new())
        // After instrumentation: probes stay out of request traces and Sentry.
        .route("/healthz", get(routes::get_healthz));
    match web_dir {
        Some(dir) => router.fallback_service(
            ServeDir::new(&dir).not_found_service(ServeFile::new(dir.join("404.html"))),
        ),
        None => router,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn web_dir_is_optional_in_debug_builds_only() {
        let unset = parse_web_dir(None);
        let empty = parse_web_dir(Some("".into()));
        if cfg!(debug_assertions) {
            assert_eq!(unset.unwrap(), None);
            assert_eq!(empty.unwrap(), None);
        } else {
            assert!(unset.is_err() && empty.is_err());
        }
    }
}
