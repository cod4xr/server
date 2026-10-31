// SPDX-License-Identifier: AGPL-3.0-or-later
//! Unsigned IW3 version-5 envelope integrity, without asset graph interpretation.

mod fastfile_budget;
mod fastfile_error;
mod fastfile_header;
mod fastfile_stream;
mod inflate_cursor;

pub use fastfile_budget::FastFileBudget;
pub use fastfile_error::{FastFileError, FastFileResource};
pub use fastfile_header::{FastFileHeader, IW3_BLOCK_COUNT};
pub use fastfile_stream::FastFileStream;
