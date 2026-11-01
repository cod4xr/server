// SPDX-License-Identifier: AGPL-3.0-or-later
//! Offline execution does not fabricate a world when source accesses special roots.
#![allow(
    clippy::expect_used,
    reason = "Tests execute bounded real offline root programs and inspect private structured/human diagnostics."
)]
mod content_support;

use std::process::Command;

use content_support::FixtureFile;

#[test]
fn executed_roots_fail_in_the_execution_phase_without_world_handles_or_private_source() {
    for root in ["level", "self", "game", "anim"] {
        let source = format!("main(){{/* PrivateRootPayload */return {root};}}");
        let file = FixtureFile::from_bytes(source.as_bytes());
        let output = Command::new(env!("CARGO_BIN_EXE_cod4x-server"))
            .args(["--json-logs", "--execute-gsc"])
            .arg(&file.0)
            .output()
            .expect("Offline missing-root process");
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        let logs = String::from_utf8(output.stderr).expect("Private JSON logs");
        assert!(logs.contains("ScriptRootContextRequired"));
        assert!(logs.contains("\"phase\":\"execute\""));
        assert!(!logs.contains("PrivateRootPayload"));
        assert!(!logs.contains("ObjectId"));
        assert!(!logs.contains("ArrayValueId"));
        assert!(!logs.contains(file.0.to_string_lossy().as_ref()));
        let records: Vec<serde_json::Value> = logs
            .lines()
            .filter(|line| !line.starts_with("cod4x-server: "))
            .map(|line| serde_json::from_str(line).expect("Complete JSON event"))
            .collect();
        let fields = &records
            .iter()
            .find(|record| record["fields"]["event"] == "script.failed")
            .expect("Failure event")["fields"];
        assert_eq!(fields["cause"], "ScriptRootContextRequired");
        assert_eq!(fields["line"], 1);
        assert_eq!(
            fields["offset"],
            source.rfind(root).expect("Root spelling") as u64
        );
        assert_eq!(
            fields["byte_column"],
            source.rfind(root).expect("Root spelling") as u64 + 1
        );
    }
}

#[test]
fn skipped_root_access_needs_no_world_and_preserves_readable_development_logs() {
    let file = FixtureFile::from_bytes(b"main(){if(0)return game;return 7;}");
    let output = Command::new(env!("CARGO_BIN_EXE_cod4x-server"))
        .args(["--color", "never", "--execute-gsc"])
        .arg(&file.0)
        .output()
        .expect("Skipped root process");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"integer 7\n");
    let logs = String::from_utf8(output.stderr).expect("Readable logs");
    assert!(logs.contains("script.executed"));
    assert!(!logs.contains("ScriptRootContextRequired"));
    assert!(!logs.contains(file.0.to_string_lossy().as_ref()));
    assert!(!logs.contains('\u{1b}'));
}
