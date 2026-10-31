// SPDX-License-Identifier: AGPL-3.0-or-later
//! Process entry point for the explicit query-only development runtime.

mod app;
mod app_error;
mod arguments;
mod counters;
mod runtime;

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
