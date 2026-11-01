// SPDX-License-Identifier: AGPL-3.0-or-later
//! Process entry point for the explicit query-only development runtime.

mod app;
mod app_error;
mod arguments;
mod content;
mod content_snapshot;
mod counters;
mod ready_socket;
mod runtime;
mod script;
mod script_execution;
mod script_result;
mod script_runtime;
mod service;

use std::process::ExitCode;

fn main() -> ExitCode {
    match app::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("cod4x-server: {error}");
            ExitCode::FAILURE
        }
    }
}
