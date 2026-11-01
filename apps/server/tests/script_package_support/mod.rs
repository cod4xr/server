// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Helpers own synthetic files and real offline child processes."
)]
use std::process::{Command, Output};

use crate::content_support::FixtureFile;

pub fn execute(entry: &str, inputs: &[(&str, &FixtureFile)], options: &[&str]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_cod4x-server"));
    command.args(["--execute-gsc-package", entry]);
    for (name, file) in inputs {
        command.args(["--gsc-module", name]).arg(&file.0);
    }
    command.args(options).output().expect("Package process")
}

pub fn logs(output: &Output, inputs: &[&FixtureFile]) -> String {
    let logs = String::from_utf8(output.stderr.clone()).expect("Logs");
    assert!(!logs.contains("Private"));
    assert!(!logs.contains("server.started"));
    for file in inputs {
        assert!(!logs.contains(file.0.to_string_lossy().as_ref()));
    }
    logs
}

pub fn failed(output: &Output, inputs: &[&FixtureFile]) -> serde_json::Value {
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    logs(output, inputs)
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .find(|record| record["fields"]["event"] == "script.failed")
        .expect("Located failed event")
}
