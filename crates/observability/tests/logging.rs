// SPDX-License-Identifier: AGPL-3.0-or-later
//! File flush, JSON schema, filtering, and isolated subscriber behavior.
#![allow(
    clippy::expect_used,
    reason = "Tests fail immediately with setup context."
)]

use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

use cod4x_observability::{ConsoleFormat, LogConfig, LogRuntime};
use serde_json::Value;

static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

#[test]
fn rotating_file_output_is_json_filtered_and_flushed_on_shutdown() {
    let unique = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
    let directory = std::env::temp_dir().join(format!(
        "cod4x-logging-test-{}-{unique}",
        std::process::id()
    ));
    fs::create_dir(&directory).expect("Unique temporary test directory");
    let mut config = LogConfig::default();
    config.directory = Some(directory.clone());
    config.console_format = ConsoleFormat::Human;
    config.filter = String::from("warn");
    let runtime = LogRuntime::build(&config).expect("Writable log directory");
    tracing::dispatcher::with_default(runtime.dispatch(), || {
        tracing::info!(event = "filtered_event", "must be filtered");
        let span = tracing::warn_span!("connection", connection_id = 7_u64);
        let _entered = span.enter();
        tracing::warn!(
            event = "connection.closed",
            reason = "timeout",
            "connection closed"
        );
    });
    assert_eq!(runtime.dropped_records(), 0);
    drop(runtime);
    let files: Vec<_> = fs::read_dir(&directory)
        .expect("Read test directory")
        .map(|entry| entry.expect("Directory entry").path())
        .collect();
    assert_eq!(files.len(), 1, "Only the owned daily file should exist");
    let contents = fs::read_to_string(&files[0]).expect("Flushed UTF-8 log");
    let lines: Vec<_> = contents.lines().collect();
    assert_eq!(lines.len(), 1, "Info-level event must have been filtered");
    let event: Value = serde_json::from_str(lines[0]).expect("Each log record is JSON");
    assert_eq!(event["level"], "WARN");
    assert_eq!(event["fields"]["event"], "connection.closed");
    assert_eq!(event["fields"]["reason"], "timeout");
    assert_eq!(event["span"]["connection_id"], 7);
    assert!(
        event["timestamp"]
            .as_str()
            .is_some_and(|stamp| stamp.ends_with('Z'))
    );
    assert!(!contents.contains("filtered_event"));
    for file in files {
        fs::remove_file(file).expect("Remove only test-owned file");
    }
    fs::remove_dir(directory).expect("Remove empty test-owned directory");
}

#[test]
fn invalid_limits_and_filters_fail_before_application_startup() {
    for (buffered, retained) in [(0, 14), (65_537, 14), (1, 0), (1, 366)] {
        let mut config = LogConfig::default();
        config.buffered_records = buffered;
        config.retained_files = retained;
        assert!(LogRuntime::build(&config).is_err());
    }
    let mut config = LogConfig::default();
    config.filter = String::from("module=not_a_level");
    assert!(LogRuntime::build(&config).is_err());
}
