// SPDX-License-Identifier: AGPL-3.0-or-later
use std::io;

use tracing::Dispatch;
use tracing_appender::non_blocking::{ErrorCounter, NonBlocking, NonBlockingBuilder, WorkerGuard};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::{Layer, SubscriberExt};
use tracing_subscriber::registry::Registry;

use crate::human_timer::HumanTimer;
use crate::{ConsoleColor, ConsoleFormat, LogConfig, LogError};

type OutputLayer = Box<dyn Layer<Registry> + Send + Sync>;

/// Owns a dispatcher and its bounded nonblocking output workers.
///
/// Dropping flushes pending output subject to the appender's bounded shutdown
/// timeout. Abrupt process termination cannot guarantee log delivery. Sink I/O
/// failures and queue overflow require external monitoring as well as this
/// runtime's dropped-record counters; logs are not a durable transaction store.
pub struct LogRuntime {
    dispatch: Dispatch,
    counters: Vec<ErrorCounter>,
    _guards: Vec<WorkerGuard>,
}

impl LogRuntime {
    /// Builds sinks and an isolated dispatcher without setting a global default.
    ///
    /// The supplied directory must be dedicated to application logs: retention
    /// deletes old files matching the `cod4x` prefix and `jsonl` suffix. Regex
    /// matching is disabled in filter directives. Queue capacity is 1..=65536
    /// records per sink and retention is 1..=365 files.
    ///
    /// # Errors
    /// Rejects invalid filters/limits and reports file initialization errors.
    pub fn build(config: &LogConfig) -> Result<Self, LogError> {
        if !(1..=65_536).contains(&config.buffered_records)
            || !(1..=365).contains(&config.retained_files)
            || config.filter.len() > 1024
        {
            return Err(LogError::InvalidLimits);
        }
        let filter = EnvFilter::builder()
            .with_regex(false)
            .parse(&config.filter)
            .map_err(LogError::InvalidFilter)?;
        let mut layers: Vec<OutputLayer> = Vec::new();
        let mut counters = Vec::new();
        let mut guards = Vec::new();
        let (console, guard) = Self::queue(io::stderr(), config.buffered_records);
        counters.push(console.error_counter());
        guards.push(guard);
        let console_layer: OutputLayer = match (config.console_format, cfg!(debug_assertions)) {
            (ConsoleFormat::Json, _) | (ConsoleFormat::Auto, false) => fmt::layer()
                .json()
                .with_ansi(false)
                .with_current_span(true)
                .with_span_list(true)
                .with_writer(console)
                .boxed(),
            (ConsoleFormat::Human, _) | (ConsoleFormat::Auto, true) => fmt::layer()
                .compact()
                .with_target(false)
                .with_timer(HumanTimer::default())
                .with_ansi(Self::color_enabled(config.console_color))
                .with_writer(console)
                .boxed(),
        };
        layers.push(console_layer);
        if let Some(directory) = &config.directory {
            let appender = RollingFileAppender::builder()
                .rotation(Rotation::DAILY)
                .filename_prefix("cod4x")
                .filename_suffix("jsonl")
                .max_log_files(config.retained_files)
                .build(directory)
                .map_err(LogError::FileSetup)?;
            let (file, guard) = Self::queue(appender, config.buffered_records);
            counters.push(file.error_counter());
            guards.push(guard);
            layers.push(
                fmt::layer()
                    .json()
                    .with_ansi(false)
                    .with_current_span(true)
                    .with_span_list(true)
                    .with_writer(file)
                    .boxed(),
            );
        }
        let subscriber = tracing_subscriber::registry().with(layers).with(filter);
        Ok(Self {
            dispatch: Dispatch::new(subscriber),
            counters,
            _guards: guards,
        })
    }

    /// Returns the dispatcher for application installation or scoped test use.
    #[must_use]
    pub fn dispatch(&self) -> &Dispatch {
        &self.dispatch
    }

    /// Returns the total records dropped by full output queues since startup.
    ///
    /// This measures queue overflow, not every possible downstream I/O failure.
    #[must_use]
    pub fn dropped_records(&self) -> usize {
        self.counters.iter().fold(0, |total, counter| {
            total.saturating_add(counter.dropped_lines())
        })
    }

    fn queue(writer: impl io::Write + Send + 'static, limit: usize) -> (NonBlocking, WorkerGuard) {
        NonBlockingBuilder::default()
            .buffered_lines_limit(limit)
            .lossy(true)
            .finish(writer)
    }

    fn color_enabled(policy: ConsoleColor) -> bool {
        use std::io::IsTerminal;

        match policy {
            ConsoleColor::Always => true,
            ConsoleColor::Never => false,
            ConsoleColor::Auto => {
                io::stderr().is_terminal()
                    && std::env::var_os("NO_COLOR").is_none()
                    && std::env::var_os("TERM").is_none_or(|term| term != "dumb")
            }
        }
    }
}
