// SPDX-License-Identifier: AGPL-3.0-or-later
use std::path::Path;

use cod4x_content::fastfile::{FastFileBudget, FastFileStream};
use cod4x_content::{ArchiveBudget, IwdArchive};

use crate::app_error::AppError;
use crate::content_snapshot;

pub(crate) fn verify_iwd(path: &Path) -> Result<(), AppError> {
    tracing::info!(
        event = "content.started",
        mode = "iwd_verification",
        "verifying content archive"
    );
    let result = verify(path);
    report_failure(&result);
    result
}

pub(crate) fn inspect_fastfile(path: &Path) -> Result<(), AppError> {
    tracing::info!(
        event = "content.started",
        mode = "fastfile_inspection",
        "inspecting fastfile envelope"
    );
    let result = inspect(path);
    report_failure(&result);
    result
}

fn report_failure(result: &Result<(), AppError>) {
    if let Err(error) = result {
        match error {
            AppError::Content(error) => {
                tracing::error!(event = "content.failed", kind = ?error,
                    "content verification failed");
            }
            AppError::FastFile(error) => {
                tracing::error!(event = "content.failed", kind = ?error,
                    "fastfile envelope validation failed");
            }
            AppError::Snapshot(error) => {
                tracing::error!(
                    event = "content.failed",
                    kind = error.category(),
                    "content snapshot could not be read"
                );
            }
            _ => {}
        }
    }
}

fn verify(path: &Path) -> Result<(), AppError> {
    let budget = ArchiveBudget::default();
    let bytes =
        content_snapshot::read(path, budget.max_archive_bytes).map_err(AppError::Snapshot)?;
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

fn inspect(path: &Path) -> Result<(), AppError> {
    let budget = FastFileBudget::default();
    let bytes = content_snapshot::read(path, budget.max_file_bytes).map_err(AppError::Snapshot)?;
    let stream = FastFileStream::decode(&bytes, budget).map_err(AppError::FastFile)?;
    let header = stream.header();
    tracing::info!(
        event = "content.envelope_verified",
        encoded_bytes = bytes.len(),
        payload_bytes = header.payload_bytes(),
        external_bytes = header.external_bytes(),
        total_block_bytes = header.total_block_bytes(),
        "fastfile envelope verified; asset graph remains unvalidated"
    );
    Ok(())
}
