// SPDX-License-Identifier: AGPL-3.0-or-later
use std::error::Error;
use std::fmt;

use tracing_appender::rolling::InitError;
use tracing_subscriber::filter::ParseError;

/// A logging configuration failure; applications must report it before starting.
#[derive(Debug)]
#[non_exhaustive]
pub enum LogError {
    /// A queue, retention, or filter-size setting is outside the supported bounds.
    InvalidLimits,
    /// Explicit filter directives are invalid; they are not silently ignored.
    InvalidFilter(ParseError),
    /// The configured file sink cannot be opened or initialized.
    FileSetup(InitError),
}

impl fmt::Display for LogError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLimits => {
                formatter.write_str("logging limits are outside supported bounds")
            }
            Self::InvalidFilter(_) => formatter.write_str("invalid logging filter directives"),
            Self::FileSetup(error) => write!(formatter, "cannot initialize log files: {error}"),
        }
    }
}

impl Error for LogError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidLimits => None,
            Self::InvalidFilter(error) => Some(error),
            Self::FileSetup(error) => Some(error),
        }
    }
}
