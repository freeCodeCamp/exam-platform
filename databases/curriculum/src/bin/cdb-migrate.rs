//! cdb-migrate: applies cDb's pending migrations to branch `dev`, one Dolt commit each, then exits.
//! Migrates on scratch branch `cdb-migrate` and merges it into `dev`, so a failed run leaves `dev`
//! untouched. Never touches other branches: they get the schema when `dev` is merged into them.
//! Ships in eDd's image only; deploys run it as a one-shot container before replacing eDd
//! (deploy/exam-platform.toml). Exit 0 only once every migration this build embeds is on `dev`.
//!
//! Env: `CDB_MIGRATOR_URL`, naming the database (`.../curriculum`), never a `db/branch` revision.
use std::{process::ExitCode, time::Duration};

/// Bounds the whole run: waiting for another run's lock, then applying and merging. The deploy
/// Action's MIGRATE_SECONDS must stay above this plus the image pull. A run cut short ends its
/// session, which releases the lock; anything it left is on the scratch branch, which the next run
/// recreates.
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
        Ok(Ok(applied)) if applied.is_empty() => {
            println!("cDb up to date");
            ExitCode::SUCCESS
        }
        Ok(Ok(applied)) => {
            println!("cDb migrations applied to {}: {applied:?}", cdb::DEV);
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
