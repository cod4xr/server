// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;
use std::ops::Range;

use crate::source::source_error;
use crate::source::{SourceBudget, SourceError, SourceLine, SourceLocation, SourceResource};

/// Immutable source plus an owned bounded line-start index, without host paths.
///
/// Raw bytes, tabs, NULs, and non-UTF-8 data stay unchanged. This is a frontend/
/// tooling container, not lexical validation or a module registry. LF starts the
/// next line; CRLF is preserved and standalone CR is ordinary content. EOF is a
/// valid location; an empty source has one empty line. No globals are installed.
pub struct SourceText<'source> {
    bytes: &'source [u8],
    starts: Vec<usize>,
}

impl<'source> SourceText<'source> {
    /// Admits source/line/metadata budgets before fallible index allocation.
    ///
    /// # Errors
    /// Rejects budget/size/allocation failures without publishing a partial index
    /// or mutating source. Metadata is logical storage, not allocator overhead/RSS.
    pub fn index(bytes: &'source [u8], budget: SourceBudget) -> Result<Self, SourceError> {
        source_error::check(
            SourceResource::SourceBytes,
            bytes.len() as u64,
            budget.max_source_bytes,
        )?;
        let lines = bytes
            .split(|byte| *byte == b'\n')
            .try_fold(0_usize, |count, _| {
                count.checked_add(1).ok_or(SourceError::SizeOverflow)
            })?;
        source_error::check(SourceResource::Lines, lines as u64, budget.max_lines)?;
        let metadata = lines
            .checked_mul(size_of::<usize>())
            .ok_or(SourceError::SizeOverflow)?;
        source_error::check(
            SourceResource::MetadataBytes,
            metadata as u64,
            budget.max_metadata_bytes,
        )?;
        let mut starts = Vec::new();
        starts
            .try_reserve_exact(lines)
            .map_err(|_| SourceError::AllocationFailed)?;
        starts.push(0);
        for (offset, byte) in bytes.iter().enumerate() {
            if *byte == b'\n' {
                starts.push(offset + 1);
            }
        }
        Ok(Self { bytes, starts })
    }

    /// Borrows all original bytes without normalization or an implicit terminator.
    #[must_use]
    pub fn bytes(&self) -> &'source [u8] {
        self.bytes
    }

    /// Returns line count, including the empty final line following a trailing LF.
    #[must_use]
    pub fn line_count(&self) -> u64 {
        self.starts.len() as u64
    }

    /// Returns logical line-index storage, excluding source/allocator overhead.
    #[must_use]
    pub fn metadata_bytes(&self) -> u64 {
        (self.starts.len() * size_of::<usize>()) as u64
    }

    /// Maps a byte offset through EOF to one-based line/byte-column coordinates.
    ///
    /// # Errors
    /// Rejects offsets beyond this source; does not clamp or truncate them.
    pub fn location(&self, offset: u64) -> Result<SourceLocation, SourceError> {
        let native = usize::try_from(offset).map_err(|_| SourceError::InvalidOffset)?;
        if native > self.bytes.len() {
            return Err(SourceError::InvalidOffset);
        }
        // Every index contains start zero, and a validated native offset is >= 0.
        let line = self.starts.partition_point(|start| *start <= native) - 1;
        Ok(SourceLocation {
            offset,
            line: line as u64 + 1,
            byte_column: (native - self.starts[line]) as u64 + 1,
        })
    }

    /// Borrows a one-based line without revealing it through Debug output.
    ///
    /// # Errors
    /// Rejects zero/unrepresentable/out-of-range line numbers.
    pub fn line(&self, number: u64) -> Result<SourceLine<'source>, SourceError> {
        let index = number
            .checked_sub(1)
            .and_then(|line| usize::try_from(line).ok())
            .ok_or(SourceError::InvalidLine)?;
        let start = *self.starts.get(index).ok_or(SourceError::InvalidLine)?;
        let end = self
            .starts
            .get(index + 1)
            .copied()
            .unwrap_or(self.bytes.len());
        Ok(SourceLine::new(start as u64, &self.bytes[start..end]))
    }

    /// Borrows an exact half-open byte span, allowing empty spans through EOF.
    ///
    /// # Errors
    /// Rejects reversed, unrepresentable, or out-of-bounds spans without repair.
    pub fn span(&self, range: Range<u64>) -> Result<&'source [u8], SourceError> {
        let start = usize::try_from(range.start).map_err(|_| SourceError::InvalidSpan)?;
        let end = usize::try_from(range.end).map_err(|_| SourceError::InvalidSpan)?;
        self.bytes.get(start..end).ok_or(SourceError::InvalidSpan)
    }
}

impl fmt::Debug for SourceText<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SourceText")
            .field("bytes", &self.bytes.len())
            .field("lines", &self.starts.len())
            .finish_non_exhaustive()
    }
}
