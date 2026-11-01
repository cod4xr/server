// SPDX-License-Identifier: AGPL-3.0-or-later
use std::error::Error;
use std::fmt;
use std::io;

use crate::content_snapshot::SnapshotError;

#[derive(Debug)]
pub(crate) enum AppError {
    Arguments(&'static str),
    Logging(cod4x_observability::LogError),
    Subscriber(tracing::subscriber::SetGlobalDefaultError),
    Signal(ctrlc::Error),
    Io(io::Error),
    Content(cod4x_content::ArchiveError),
    FastFile(cod4x_content::fastfile::FastFileError),
    ZonePrefix(cod4x_content::zone::ZonePrefixError),
    RawFile(cod4x_content::zone::rawfiles::RawFileError),
    ScriptSource(cod4x_script::source::SourceError),
    ScriptSyntax(cod4x_script::syntax::LegacySyntaxError),
    ScriptDeclaration(cod4x_script::declaration::DeclarationError),
    ScriptBinding(cod4x_script::binding::SymbolError),
    ScriptCompilation(cod4x_script::compilation::ModuleCompileError),
    ScriptExecution(cod4x_script::execution::KernelModuleError),
    ScriptFeature(&'static str),
    ScriptOutput(io::Error),
    Snapshot(SnapshotError),
    Entropy(getrandom::Error),
    Cookie(cod4x_server_core::CookieKeyError),
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
            Self::FastFile(error) => write!(formatter, "fastfile inspection failed: {error}"),
            Self::ZonePrefix(error) => write!(formatter, "zone prefix inspection failed: {error}"),
            Self::RawFile(error) => write!(formatter, "raw-file inspection failed: {error}"),
            Self::ScriptSource(error) => write!(formatter, "script inspection failed: {error}"),
            Self::ScriptSyntax(error) => write!(formatter, "script inspection failed: {error}"),
            Self::ScriptDeclaration(error) => {
                write!(formatter, "script inspection failed: {error}")
            }
            Self::ScriptBinding(error) => write!(formatter, "script inspection failed: {error}"),
            Self::ScriptCompilation(error) => {
                write!(formatter, "script compilation failed: {error}")
            }
            Self::ScriptExecution(error) => write!(formatter, "script execution failed: {error}"),
            Self::ScriptFeature(message) => {
                write!(formatter, "primitive script execution: {message}")
            }
            Self::ScriptOutput(error) => write!(formatter, "script result output failed: {error}"),
            Self::Snapshot(error) => error.fmt(formatter),
            Self::Entropy(error) => write!(formatter, "process entropy failed: {error}"),
            Self::Cookie(error) => write!(formatter, "cookie initialization failed: {error}"),
        }
    }
}

impl Error for AppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Arguments(_) | Self::ScriptFeature(_) => None,
            Self::Logging(error) => Some(error),
            Self::Subscriber(error) => Some(error),
            Self::Signal(error) => Some(error),
            Self::Io(error) | Self::ScriptOutput(error) => Some(error),
            Self::Content(error) => Some(error),
            Self::FastFile(error) => Some(error),
            Self::ZonePrefix(error) => Some(error),
            Self::RawFile(error) => Some(error),
            Self::ScriptSource(error) => Some(error),
            Self::ScriptSyntax(error) => Some(error),
            Self::ScriptDeclaration(error) => Some(error),
            Self::ScriptBinding(error) => Some(error),
            Self::ScriptCompilation(error) => Some(error),
            Self::ScriptExecution(error) => Some(error),
            Self::Snapshot(error) => Some(error),
            Self::Entropy(error) => Some(error),
            Self::Cookie(error) => Some(error),
        }
    }
}

impl From<io::Error> for AppError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}
