use super::*;
use crate::{
    event_handlers::{Command, StatsHandle, StatsHandler},
    CommandSender, FeroxSerialize, Joiner,
};
use anyhow::Result;
use reqwest::StatusCode;
use tempfile::NamedTempFile;

/// simple helper to reduce code reuse
pub fn setup_stats_test() -> (Joiner, StatsHandle) {
    StatsHandler::initialize()
}

/// another helper to stay DRY; must be called after any sent commands and before any checks
/// performed against the Stats object
pub async fn teardown_stats_test(sender: CommandSender, task: Joiner) {
    // send exit and await, once the await completes, stats should be updated
    sender.send(Command::Exit).unwrap_or_default();
    task.await.unwrap().unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
/// when sent StatCommand::Exit, function should exit its while loop (runs forever otherwise)
async fn statistics_handler_exits() -> Result<()> {
    let (task, handle) = setup_stats_test();

    handle.tx.send(Command::Exit)?;

    task.await??; // blocks on the handler's while loop

    // if we've made it here, the test has succeeded
    Ok(())
}

#[test]
/// Stats::save should write contents of Stats to disk
fn save_writes_stats_object_to_disk() {
    let stats = Stats::new();
    stats.add_request();
    stats.add_request();
    stats.add_request();
    stats.add_request();
    stats.add_error(StatError::Timeout);
    stats.add_error(StatError::Timeout);
    stats.add_error(StatError::Timeout);
    stats.add_error(StatError::Timeout);
    stats.add_status_code(StatusCode::OK);
    stats.add_status_code(StatusCode::OK);
    stats.add_status_code(StatusCode::OK);
    let outfile = NamedTempFile::new().unwrap();
    if stats
        .save(174.33, &outfile.path().to_str().unwrap())
        .is_ok()
    {}

    assert!(stats.as_json().unwrap().contains("statistics"));
    assert!(stats.as_json().unwrap().contains("11")); // requests made
    assert!(stats.as_str().is_empty());
}