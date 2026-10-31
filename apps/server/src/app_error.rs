// SPDX-License-Identifier: AGPL-3.0-or-later
use std::error::Error;
use std::fmt;
use std::io;

#[derive(Debug)]
pub(crate) enum AppError {
    Arguments(&'static str),
    Logging(cod4x_observability::LogError),
    Subscriber(tracing::subscriber::SetGlobalDefaultError),
    Signal(ctrlc::Error),
    Io(io::Error),
    Content(cod4x_content::ArchiveError),
    ContentIo(io::Error),
}

impl fmt::Display for AppError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Arguments(message) => formatter.write_str(message),
            Self::Logging(error) => write!(formatter, "logging setup failed: {error}"),
            Self::Subscriber(error) => write!(formatter, "logging installation failed: {error}"),
            Self::Signal(error) => {
                write!(formatter, "shutdown signal registration failed: {error}")
            }
            Self::Io(error) => write!(formatter, "socket operation failed: {error}"),
            Self::Content(error) => write!(formatter, "content verification failed: {error}"),
            Self::ContentIo(error) => write!(formatter, "content read failed: {error}"),
        }
    }
}

impl Error for AppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Arguments(_) => None,
            Self::Logging(error) => Some(error),
            Self::Subscriber(error) => Some(error),
            Self::Signal(error) => Some(error),
            Self::Io(error) | Self::ContentIo(error) => Some(error),
            Self::Content(error) => Some(error),
        }
    }
}

impl From<io::Error> for AppError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}
