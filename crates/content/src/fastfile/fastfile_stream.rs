// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use crate::fastfile::fastfile_header::{self, XFILE_HEADER_BYTES};
use crate::fastfile::inflate_cursor::InflateCursor;
use crate::fastfile::{FastFileBudget, FastFileError, FastFileHeader, FastFileResource};

const CHUNK_BYTES: usize = 16 * 1024;

/// Owned, integrity-checked serialized asset stream from an unsigned v5 envelope.
///
/// Payload bytes still contain unvalidated asset records/pointer encodings. This
/// type does not load a zone, allocate declared blocks, resolve asset graphs,
/// authenticate content, read external images, or publish a playable map.
pub struct FastFileStream {
    header: FastFileHeader,
    payload: Vec<u8>,
}

impl FastFileStream {
    /// Checks headers/budgets and decompresses exactly one complete zlib stream.
    ///
    /// Decodes the fixed header before growing output storage. Output grows
    /// fallibly in bounded chunks/geometric capacities; a forged huge declaration
    /// does not cause eager full-size allocation. Decoder workspace is fixed-size
    /// but its dependency allocations are not universally fallible.
    ///
    /// # Errors
    /// Rejects invalid/unsupported headers, signed envelopes, budget/size failures,
    /// corrupt/incomplete output, and any compressed trailer. No partial output is
    /// returned. This establishes compression integrity, not asset graph validity.
    pub fn decode(bytes: &[u8], budget: FastFileBudget) -> Result<Self, FastFileError> {
        fastfile_header::check(
            FastFileResource::FileBytes,
            bytes.len() as u64,
            budget.max_file_bytes,
        )?;
        let outer = bytes.get(..12).ok_or(FastFileError::Truncated)?;
        if &outer[..8] != b"IWffu100" && &outer[..8] != b"IWff0100" {
            return Err(FastFileError::InvalidMagic);
        }
        if u32::from_le_bytes([outer[8], outer[9], outer[10], outer[11]]) != 5 {
            return Err(FastFileError::UnsupportedVersion);
        }
        if &outer[..8] == b"IWff0100" {
            return Err(FastFileError::UnsupportedAuthentication);
        }
        let mut input = InflateCursor::new(&bytes[12..]);
        let mut header_bytes = [0; XFILE_HEADER_BYTES];
        input.read_exact(&mut header_bytes)?;
        let header = FastFileHeader::parse(&header_bytes, budget)?;
        let expected =
            usize::try_from(header.payload_bytes()).map_err(|_| FastFileError::SizeOverflow)?;
        let mut payload = Vec::new();
        let mut scratch = [0; CHUNK_BYTES];
        while payload.len() < expected {
            let chunk = (expected - payload.len()).min(scratch.len());
            input.read_exact(&mut scratch[..chunk])?;
            reserve(&mut payload, chunk, expected)?;
            payload.extend_from_slice(&scratch[..chunk]);
        }
        input.finish()?;
        Ok(Self { header, payload })
    }

    /// Returns admitted allocation/accounting metadata without interpreting assets.
    #[must_use]
    pub fn header(&self) -> FastFileHeader {
        self.header
    }

    /// Borrows the serialized asset stream, excluding the fixed `XFile` header.
    #[must_use]
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    /// Transfers the decoded serialized bytes without cloning or resolving pointers.
    #[must_use]
    pub fn into_payload(self) -> Vec<u8> {
        self.payload
    }
}

fn reserve(payload: &mut Vec<u8>, chunk: usize, expected: usize) -> Result<(), FastFileError> {
    let needed = payload
        .len()
        .checked_add(chunk)
        .ok_or(FastFileError::SizeOverflow)?;
    if needed > payload.capacity() {
        let target = expected.min(
            payload
                .capacity()
                .saturating_mul(2)
                .max(CHUNK_BYTES)
                .max(needed),
        );
        payload
            .try_reserve_exact(target - payload.len())
            .map_err(|_| FastFileError::AllocationFailed)?;
    }
    Ok(())
}

impl fmt::Debug for FastFileStream {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FastFileStream")
            .field("header", &self.header)
            .field("payload_bytes", &self.payload.len())
            .finish_non_exhaustive()
    }
}
