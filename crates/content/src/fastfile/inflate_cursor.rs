// SPDX-License-Identifier: AGPL-3.0-or-later
use flate2::{Decompress, FlushDecompress, Status};

use crate::fastfile::FastFileError;

/// Bounded in-memory zlib progress, with explicit end/checksum/trailer validation.
pub(super) struct InflateCursor<'input> {
    input: &'input [u8],
    decoder: Decompress,
    ended: bool,
}

impl<'input> InflateCursor<'input> {
    pub(super) fn new(input: &'input [u8]) -> Self {
        Self {
            input,
            decoder: Decompress::new(true),
            ended: false,
        }
    }

    pub(super) fn read_exact(&mut self, output: &mut [u8]) -> Result<(), FastFileError> {
        let mut written = 0;
        while written < output.len() {
            if self.ended {
                return Err(FastFileError::SizeMismatch);
            }
            written += self.step(&mut output[written..], FlushDecompress::None)?;
        }
        Ok(())
    }

    pub(super) fn finish(&mut self) -> Result<(), FastFileError> {
        while !self.ended {
            if self.step(&mut [0], FlushDecompress::Finish)? != 0 {
                return Err(FastFileError::SizeMismatch);
            }
        }
        if self.decoder.total_in() != self.input.len() as u64 {
            return Err(FastFileError::TrailingData);
        }
        Ok(())
    }

    fn step(&mut self, output: &mut [u8], flush: FlushDecompress) -> Result<usize, FastFileError> {
        let before_input = self.decoder.total_in();
        let before_output = self.decoder.total_out();
        let offset = usize::try_from(before_input).map_err(|_| FastFileError::SizeOverflow)?;
        let input = self.input.get(offset..).ok_or(FastFileError::Compression)?;
        let status = self
            .decoder
            .decompress(input, output, flush)
            .map_err(|_| FastFileError::Compression)?;
        let produced = self
            .decoder
            .total_out()
            .checked_sub(before_output)
            .ok_or(FastFileError::Compression)?;
        let produced = usize::try_from(produced).map_err(|_| FastFileError::SizeOverflow)?;
        if produced > output.len() || self.decoder.total_in() > self.input.len() as u64 {
            return Err(FastFileError::Compression);
        }
        self.ended = status == Status::StreamEnd;
        if !self.ended && produced == 0 && self.decoder.total_in() == before_input {
            return Err(FastFileError::Compression);
        }
        Ok(produced)
    }
}
