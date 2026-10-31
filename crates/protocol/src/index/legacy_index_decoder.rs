// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{LegacyIndexError, LegacyIndexKind, MessageReader};

/// Validating legacy snapshot-index decoder, isolated from engine entity IDs.
#[derive(Clone, Copy, Debug)]
pub struct LegacyIndexDecoder {
    kind: LegacyIndexKind,
    previous: Option<u16>,
}

impl LegacyIndexDecoder {
    /// Creates a stream with the same -1 baseline as the legacy encoder.
    #[must_use]
    pub fn new(kind: LegacyIndexKind) -> Self {
        Self {
            kind,
            previous: None,
        }
    }

    /// Starts another ordered stream at the -1 baseline.
    pub fn reset(&mut self) {
        self.previous = None;
    }

    /// Reads and validates the next strictly increasing, representable index.
    ///
    /// Reader cursor and decoder state remain unchanged on every failure.
    ///
    /// # Errors
    /// Rejects truncated fields, out-of-range increments, and non-increasing data.
    pub fn read(&mut self, reader: &mut MessageReader<'_>) -> Result<u16, LegacyIndexError> {
        let mut candidate = reader.clone();
        let previous = self.previous.map_or(-1, i32::from);
        let index = if candidate.read_bits(1)? != 0 {
            previous + 1
        } else if self.kind == LegacyIndexKind::Entity && candidate.read_bits(1)? == 0 {
            let delta =
                i32::try_from(candidate.read_bits(4)?).map_err(|_| LegacyIndexError::OutOfRange)?;
            previous + delta
        } else {
            i32::try_from(candidate.read_bits(self.kind.bits())?)
                .map_err(|_| LegacyIndexError::OutOfRange)?
        };
        let index = u16::try_from(index).map_err(|_| LegacyIndexError::OutOfRange)?;
        if index > self.kind.maximum_index() {
            return Err(LegacyIndexError::OutOfRange);
        }
        if i32::from(index) <= previous {
            return Err(LegacyIndexError::NotIncreasing);
        }
        self.previous = Some(index);
        *reader = candidate;
        Ok(index)
    }
}
