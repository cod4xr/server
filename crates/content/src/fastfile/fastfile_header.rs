// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::fastfile::{FastFileBudget, FastFileError, FastFileResource};

/// Block-count field bound of the IW3 version-5 serialized `XFile` header.
///
/// This does not define the engine's memory-pool or asset-ID architecture.
pub const IW3_BLOCK_COUNT: usize = 9;
pub(super) const XFILE_HEADER_BYTES: usize = 8 + IW3_BLOCK_COUNT * 4;

/// Admitted decoded header values, widened independently of encoded u32 fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FastFileHeader {
    payload: u64,
    external: u64,
    blocks: [u64; IW3_BLOCK_COUNT],
}

impl FastFileHeader {
    /// Returns declared serialized payload bytes, excluding this header.
    #[must_use]
    pub fn payload_bytes(self) -> u64 {
        self.payload
    }

    /// Returns external-data accounting without claiming those assets were loaded.
    #[must_use]
    pub fn external_bytes(self) -> u64 {
        self.external
    }

    /// Returns declared allocation sizes for the profile's nine stream blocks.
    #[must_use]
    pub fn block_bytes(&self) -> &[u64; IW3_BLOCK_COUNT] {
        &self.blocks
    }

    /// Returns total declared block allocation; no blocks have been allocated here.
    #[must_use]
    pub fn total_block_bytes(self) -> u64 {
        self.blocks.iter().sum()
    }

    pub(super) fn parse(
        bytes: &[u8; XFILE_HEADER_BYTES],
        budget: FastFileBudget,
    ) -> Result<Self, FastFileError> {
        let payload_bytes = u64::from(word(bytes, 0));
        let external_bytes = u64::from(word(bytes, 4));
        check(
            FastFileResource::PayloadBytes,
            payload_bytes,
            budget.max_payload_bytes,
        )?;
        check(
            FastFileResource::ExternalBytes,
            external_bytes,
            budget.max_external_bytes,
        )?;
        let mut block_bytes = [0; IW3_BLOCK_COUNT];
        let mut total = 0_u64;
        for (index, block) in block_bytes.iter_mut().enumerate() {
            *block = u64::from(word(bytes, 8 + index * 4));
            check(FastFileResource::BlockBytes, *block, budget.max_block_bytes)?;
            total = total
                .checked_add(*block)
                .ok_or(FastFileError::SizeOverflow)?;
            check(
                FastFileResource::TotalBlockBytes,
                total,
                budget.max_total_block_bytes,
            )?;
        }
        Ok(Self {
            payload: payload_bytes,
            external: external_bytes,
            blocks: block_bytes,
        })
    }
}

fn word(bytes: &[u8; XFILE_HEADER_BYTES], offset: usize) -> u32 {
    // Only fixed, bounded header field offsets call this helper.
    u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

pub(super) fn check(
    resource: FastFileResource,
    requested: u64,
    limit: u64,
) -> Result<(), FastFileError> {
    if requested > limit {
        return Err(FastFileError::BudgetExceeded(resource));
    }
    Ok(())
}
