// SPDX-License-Identifier: AGPL-3.0-or-later
use std::path::PathBuf;

/// Console representation; files always use structured newline-delimited JSON.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ConsoleFormat {
    /// Readable development output in debug builds; JSON in release builds.
    #[default]
    Auto,
    /// Machine-readable records for production supervisors and collectors.
    Json,
    /// Compact readable text with elapsed time, severity, message, and fields.
    Human,
}

/// Color policy for readable console logs. JSON and file sinks never use ANSI.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ConsoleColor {
    /// Enable only on a terminal, respecting `NO_COLOR` and `TERM=dumb`.
    #[default]
    Auto,
    /// Force ANSI colors, including in redirected output.
    Always,
    /// Disable ANSI colors.
    Never,
}

/// Operator-owned logging settings. Remote peers never supply these values.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct LogConfig {
    /// Explicit filter directives; defaults to `info`, without reading the environment.
    pub filter: String,
    /// Standard-error representation; file records remain JSON.
    pub console_format: ConsoleFormat,
    /// ANSI policy for readable console output.
    pub console_color: ConsoleColor,
    /// Optional dedicated directory for application-owned rotating files.
    pub directory: Option<PathBuf>,
    /// Maximum queue entries per sink. Full queues drop records instead of blocking.
    pub buffered_records: usize,
    /// Maximum number of matching daily log files retained. Must be nonzero.
    pub retained_files: usize,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            filter: String::from("info"),
            console_format: ConsoleFormat::Auto,
            console_color: ConsoleColor::Auto,
            directory: None,
            buffered_records: 4096,
            retained_files: 14,
        }
    }
}
