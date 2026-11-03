//! cdb-migrate: applies cDb's pending migrations as `cdb_migrator`, then exits. Ships in both
//! backend images; deploys run it as a one-shot container before replacing the app
//! (deploy/exam-platform.toml). Exit 0 only once every migration this build embeds is applied.
//!
//! Env: `CDB_MIGRATOR_URL`.
use std::{process::ExitCode, time::Duration};

/// Bounds the whole run: waiting for another release's run, then applying. The deploy Action's
/// MIGRATE_SECONDS must stay above this plus the image pull. A run cut short ends its session, so
/// Postgres rolls back the migration in progress and releases the lock.
const TIMEOUT: Duration = Duration::from_secs(300);

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    // Empty counts as unset, as compose passes `${VAR:-}`.
    let Some(url) = std::env::var("CDB_MIGRATOR_URL")
        .ok()
        .filter(|v| !v.is_empty())
    else {
        eprintln!("configuration error: CDB_MIGRATOR_URL not set");
        return ExitCode::from(2);
    };

    match tokio::time::timeout(TIMEOUT, cdb::migrate(&url)).await {
        Ok(Ok(())) => {
            println!("cDb migrations applied");
            ExitCode::SUCCESS
        }
        Ok(Err(err)) => {
            eprintln!("cDb migration failed: {err}");
            ExitCode::FAILURE
        }
        Err(_) => {
            eprintln!("cDb migrations did not finish in {}s", TIMEOUT.as_secs());
            ExitCode::FAILURE
        }
    }
}
