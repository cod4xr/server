// SPDX-License-Identifier: AGPL-3.0-or-later
//! Structured logging configuration, sinks, lifetime, and overload accounting.
//!
//! Applications choose and install a dispatcher. Libraries emit events without
//! installing global subscribers. Keep the runtime alive until shutdown events
//! have been emitted so its workers can flush queued records on drop.

mod human_timer;
mod log_config;
mod log_error;
mod log_runtime;

pub use log_config::{ConsoleColor, ConsoleFormat, LogConfig};
pub use log_error::LogError;
pub use log_runtime::LogRuntime;
