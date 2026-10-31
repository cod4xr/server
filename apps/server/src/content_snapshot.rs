// SPDX-License-Identifier: AGPL-3.0-or-later
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

#[derive(Debug)]
pub(crate) enum SnapshotError {
    Io(io::Error),
    NotRegular,
    TooLarge,
    SizeOverflow,
    AllocationFailed,
    Changed,
}

impl SnapshotError {
    pub(crate) fn category(&self) -> &'static str {
        match self {
            Self::Io(_) => "io",
            Self::NotRegular => "not_regular",
            Self::TooLarge => "file_budget",
            Self::SizeOverflow => "size_overflow",
            Self::AllocationFailed => "allocation_failed",
            Self::Changed => "file_changed",
        }
    }
}

impl fmt::Display for SnapshotError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Do not expose paths or arbitrary platform error strings on stderr.
        write!(formatter, "content snapshot failed: {}", self.category())?;
        if let Self::Io(error) = self {
            write!(formatter, " ({:?})", error.kind())?;
        }
        Ok(())
    }
}

impl Error for SnapshotError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

pub(crate) fn read(path: &Path, maximum: u64) -> Result<Vec<u8>, SnapshotError> {
    let mut file = File::open(path).map_err(SnapshotError::Io)?;
    let metadata = file.metadata().map_err(SnapshotError::Io)?;
    if !metadata.is_file() {
        return Err(SnapshotError::NotRegular);
    }
    if metadata.len() > maximum {
        return Err(SnapshotError::TooLarge);
    }
    let size = usize::try_from(metadata.len()).map_err(|_| SnapshotError::SizeOverflow)?;
    let mut output = Vec::new();
    output
        .try_reserve_exact(size)
        .map_err(|_| SnapshotError::AllocationFailed)?;
    output.resize(size, 0);
    file.read_exact(&mut output).map_err(SnapshotError::Io)?;
    // Detect growth without reading beyond the admitted snapshot. Concurrent
    // in-place writes are not excluded; all parsers validate the completed bytes.
    if file.read(&mut [0]).map_err(SnapshotError::Io)? != 0 {
        return Err(SnapshotError::Changed);
    }
    Ok(output)
}
