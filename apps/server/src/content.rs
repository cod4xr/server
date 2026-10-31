// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fs::File;
use std::io::Read;
use std::path::Path;

use cod4x_content::{ArchiveBudget, ArchiveError, ArchiveResource, IwdArchive};

use crate::app_error::AppError;

pub(crate) fn verify_iwd(path: &Path) -> Result<(), AppError> {
    tracing::info!(
        event = "content.started",
        mode = "iwd_verification",
        "verifying content archive"
    );
    let result = verify(path);
    if let Err(error) = &result {
        match error {
            AppError::Content(error) => {
                tracing::error!(event = "content.failed", kind = ?error,
                    "content verification failed");
            }
            AppError::ContentIo(error) => {
                tracing::error!(event = "content.failed", io_kind = ?error.kind(),
                    "content archive could not be read");
            }
            _ => {}
        }
    }
    result
}

fn verify(path: &Path) -> Result<(), AppError> {
    let budget = ArchiveBudget::default();
    let bytes = read_snapshot(path, budget.max_archive_bytes)?;
    let archive = IwdArchive::open(&bytes, budget).map_err(AppError::Content)?;
    archive.verify_all().map_err(AppError::Content)?;
    let usage = archive.usage();
    tracing::info!(
        event = "content.verified",
        entries = usage.entries,
        files = usage.files,
        archive_bytes = usage.archive_bytes,
        metadata_bytes = usage.metadata_bytes,
        declared_bytes = usage.declared_bytes,
        checksum = archive.legacy_checksum(0),
        "content archive verified"
    );
    Ok(())
}

fn read_snapshot(path: &Path, maximum: u64) -> Result<Vec<u8>, AppError> {
    let mut file = File::open(path).map_err(AppError::ContentIo)?;
    let metadata = file.metadata().map_err(AppError::ContentIo)?;
    if !metadata.is_file() {
        return Err(AppError::Content(ArchiveError::InvalidArchive));
    }
    if metadata.len() > maximum {
        return Err(AppError::Content(ArchiveError::BudgetExceeded {
            resource: ArchiveResource::ArchiveBytes,
        }));
    }
    let size = usize::try_from(metadata.len())
        .map_err(|_| AppError::Content(ArchiveError::SizeOverflow))?;
    let mut output = Vec::new();
    output
        .try_reserve_exact(size)
        .map_err(|_| AppError::Content(ArchiveError::AllocationFailed))?;
    output.resize(size, 0);
    file.read_exact(&mut output).map_err(AppError::ContentIo)?;
    // Reject a concurrently growing file instead of reading beyond the snapshot
    // budget. All subsequent parsing uses the immutable completed byte snapshot.
    if file.read(&mut [0]).map_err(AppError::ContentIo)? != 0 {
        return Err(AppError::Content(ArchiveError::InvalidArchive));
    }
    Ok(output)
}
