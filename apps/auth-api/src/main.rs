//! aAPI: candidate identity and the attempt path. The only application candidate devices reach.
mod app;
mod routes;

use runtime::{Exit, ServerConfig};

fn main() -> Exit {
    let config = match ServerConfig::from_env("127.0.0.1:13001") {
        Ok(config) => config,
        Err(err) => {
            // Telemetry is not up yet: config decides where it goes.
            eprintln!("{err}");
            return Exit::Config;
        }
    };

    // Empty counts as unset, as compose passes `${VAR:-}`.
    let Some(edb_database_url) = std::env::var("EDB_DATABASE_URL")
        .ok()
        .filter(|v| !v.is_empty())
    else {
        eprintln!("configuration error: EDB_DATABASE_URL not set");
        return Exit::Config;
    };

    // Before the Tokio runtime starts, so every worker thread inherits the Sentry hub.
    let _sentry = runtime::telemetry::init(runtime::release!(), &config.telemetry);

    runtime::serve(config.listen_addr, app::app(edb_database_url))
}
