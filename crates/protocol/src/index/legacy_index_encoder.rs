// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{LegacyIndexError, LegacyIndexKind, MessageWriter};

/// Legacy ordered snapshot-index encoder with explicit stream reset state.
#[derive(Clone, Copy, Debug)]
pub struct LegacyIndexEncoder {
    kind: LegacyIndexKind,
    previous: Option<u16>,
}

impl LegacyIndexEncoder {
    /// Creates a stream with previous index -1, as after upstream's clear call.
    #[must_use]
    pub fn new(kind: LegacyIndexKind) -> Self {
        Self {
            kind,
            previous: None,
        }
    }

    /// Starts a new ordered stream, with its previous index reset to -1.
    pub fn reset(&mut self) {
        self.previous = None;
    }

    /// Writes the increment, short-delta, or full-index form used by the peer.
    ///
    /// A single primitive bit operation makes failures atomic for both writer
    /// and index state. The caller must supply a projected legacy index, never
    /// truncate an internal entity ID to fit this method's integer type.
    ///
    /// # Errors
    /// Rejects out-of-range/non-increasing indices and insufficient output.
    pub fn write(
        &mut self,
        index: u16,
        writer: &mut MessageWriter<'_>,
    ) -> Result<(), LegacyIndexError> {
        if index > self.kind.maximum_index() {
            return Err(LegacyIndexError::OutOfRange);
        }
        let delta = i32::from(index) - self.previous.map_or(-1, i32::from);
        if delta <= 0 {
            return Err(LegacyIndexError::NotIncreasing);
        }
        let (bits, width) = if delta == 1 {
            (1, 1)
        } else if self.kind == LegacyIndexKind::Entity && delta <= 15 {
            let delta = u32::try_from(delta).map_err(|_| LegacyIndexError::NotIncreasing)?;
            (delta << 2, 6)
        } else if self.kind == LegacyIndexKind::Entity {
            ((u32::from(index) << 2) | 2, 12)
        } else {
            (u32::from(index) << 1, self.kind.bits() + 1)
        };
        writer.write_bits(bits, width)?;
        self.previous = Some(index);
        Ok(())
    }
}
