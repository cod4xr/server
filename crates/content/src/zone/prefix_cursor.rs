// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::zone::{ZonePrefixError, ZonePrefixResource};

pub(super) struct PrefixCursor<'payload> {
    remaining: &'payload [u8],
}

impl<'payload> PrefixCursor<'payload> {
    pub(super) fn new(bytes: &'payload [u8]) -> Self {
        Self { remaining: bytes }
    }

    pub(super) fn remaining(&self) -> &'payload [u8] {
        self.remaining
    }

    pub(super) fn take(&mut self, size: usize) -> Result<&'payload [u8], ZonePrefixError> {
        let (bytes, remaining) = self
            .remaining
            .split_at_checked(size)
            .ok_or(ZonePrefixError::Truncated)?;
        self.remaining = remaining;
        Ok(bytes)
    }

    pub(super) fn word(&mut self) -> Result<u32, ZonePrefixError> {
        let bytes = self.take(4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    pub(super) fn string(&mut self, limit: u64) -> Result<&'payload [u8], ZonePrefixError> {
        let available = self.remaining.len();
        let size = usize::try_from(limit).unwrap_or(usize::MAX).min(available);
        let Some(end) = self.remaining[..size].iter().position(|byte| *byte == 0) else {
            return Err(if (available as u64) < limit {
                ZonePrefixError::Truncated
            } else {
                ZonePrefixError::BudgetExceeded(ZonePrefixResource::StringBytes)
            });
        };
        Ok(&self.take(end + 1)?[..end])
    }
}
