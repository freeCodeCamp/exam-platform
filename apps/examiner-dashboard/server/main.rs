//! eDd server: examiner identity, authoring, review, promotion and moderation. Serves the built client.
mod app;
mod config;
mod database;
mod error;
mod routes;

use runtime::{Exit, ServerConfig};

fn main() -> Exit {
    let server_config = match ServerConfig::from_env("127.0.0.1:13002") {
        Ok(c) => c,
        Err(e) => return e.into(),
    };

    // Before the Tokio runtime starts, so every worker thread inherits the Sentry hub.
    let _sentry = runtime::telemetry::init(runtime::release!(), &server_config.telemetry);

    let app_config = config::AppConfig::new();

    runtime::serve(server_config.listen_addr, app::app(&app_config))
}
