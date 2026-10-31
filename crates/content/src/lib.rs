// SPDX-License-Identifier: AGPL-3.0-or-later
//! Read-only IWD archive validation, bounded decompression, and virtual paths.
//!
//! Archives borrow immutable encoded storage. No API extracts files, accesses
//! host paths, or decodes game formats. Legacy checksums preserve entry order.

mod archive_budget;
mod archive_error;
mod archive_index;
mod archive_preflight;
mod archive_usage;
mod archive_verification;
pub mod bsp;
mod entry_bounds;
pub mod fastfile;
mod iwd_archive;
mod virtual_path;
pub mod zone;

pub use archive_budget::ArchiveBudget;
pub use archive_error::{ArchiveError, ArchiveResource};
pub use archive_usage::ArchiveUsage;
pub use iwd_archive::IwdArchive;
pub use virtual_path::VirtualPath;
