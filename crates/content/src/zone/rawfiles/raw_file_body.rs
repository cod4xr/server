// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::fastfile::FastFileHeader;
use crate::zone::prefix_cursor::PrefixCursor;
use crate::zone::rawfiles::raw_file_error;
use crate::zone::rawfiles::raw_file_registry::RawFileRegistry;
use crate::zone::rawfiles::{RawFile, RawFileBudget, RawFileError, RawFileResource, RawFileUsage};
use crate::zone::{
    ZoneAddress, ZonePointer, ZonePrefix, ZonePrefixError, ZonePrefixResource, ZoneString,
};

pub(super) struct RawFileBodyReader<'payload> {
    cursor: PrefixCursor<'payload>,
    registry: RawFileRegistry<'payload>,
    header: FastFileHeader,
    budget: RawFileBudget,
    virtual_position: u64,
    name_bytes: u64,
    buffer_bytes: u64,
    temporary_bytes: u64,
}

impl<'payload> RawFileBodyReader<'payload> {
    pub(super) fn new(
        prefix: &ZonePrefix<'payload>,
        header: FastFileHeader,
        budget: RawFileBudget,
    ) -> Result<Self, RawFileError> {
        let mut registry = RawFileRegistry::new(prefix.assets().len(), budget.max_metadata_bytes)?;
        let mut offset = (prefix.strings().len() as u64)
            .checked_mul(4)
            .ok_or(RawFileError::SizeOverflow)?;
        for slot in prefix.strings() {
            if let ZoneString::Inline(bytes) = slot {
                registry.add_name(offset, bytes)?;
                offset = offset
                    .checked_add(bytes.len() as u64 + 1)
                    .ok_or(RawFileError::SizeOverflow)?;
            }
        }
        Ok(Self {
            cursor: PrefixCursor::new(prefix.remaining()),
            registry,
            header,
            budget,
            virtual_position: prefix.virtual_bytes(),
            name_bytes: 0,
            buffer_bytes: 0,
            temporary_bytes: 0,
        })
    }

    pub(super) fn bind(
        &mut self,
        offset: u64,
        target: Option<RawFile<'payload>>,
    ) -> Result<(), RawFileError> {
        self.registry.bind(offset, target)
    }

    pub(super) fn alias(
        &self,
        address: ZoneAddress,
    ) -> Result<Option<RawFile<'payload>>, RawFileError> {
        self.registry.alias(address, self.header)
    }

    pub(super) fn is_finished(&self) -> bool {
        self.cursor.remaining().is_empty()
    }

    pub(super) fn usage(&self, root_entries: usize) -> RawFileUsage {
        RawFileUsage {
            root_entries,
            inline_name_bytes: self.name_bytes,
            inline_buffer_bytes: self.buffer_bytes,
            peak_metadata_bytes: self.registry.metadata_bytes(),
            virtual_bytes: self.virtual_position,
            temporary_bytes: self.temporary_bytes,
        }
    }

    pub(super) fn read(&mut self, insert_alias: bool) -> Result<RawFile<'payload>, RawFileError> {
        // Each raw-file asset scopes and rewinds temporary block zero. Its twelve
        // header bytes must fit, but repeated assets do not accumulate temp usage.
        if self.header.block_bytes()[0] < 12 {
            return Err(RawFileError::BlockBounds);
        }
        self.temporary_bytes = 12;
        let alias = if insert_alias {
            self.virtual_position = self
                .virtual_position
                .checked_add(3)
                .ok_or(RawFileError::SizeOverflow)?
                & !3;
            let offset = self.virtual_position;
            self.advance(4)?;
            Some(offset)
        } else {
            None
        };
        let name_word = self.cursor.word()?;
        let length = self.cursor.word()?;
        let buffer_present = self.cursor.word()? != 0;
        if length > i32::MAX as u32 {
            return Err(RawFileError::InvalidLength);
        }
        let name = self.read_name(name_word)?;
        let contents = self.read_buffer(length, buffer_present)?;
        let file = RawFile { name, contents };
        if let Some(offset) = alias {
            self.registry.bind(offset, Some(file))?;
        }
        Ok(file)
    }

    fn read_name(&mut self, raw: u32) -> Result<&'payload [u8], RawFileError> {
        let name = match ZonePointer::decode(raw)? {
            ZonePointer::Inline => {
                let offset = self.virtual_position;
                let name = self
                    .cursor
                    .string(self.budget.max_name_bytes)
                    .map_err(|error| match error {
                        ZonePrefixError::BudgetExceeded(ZonePrefixResource::StringBytes) => {
                            RawFileError::BudgetExceeded(RawFileResource::NameBytes)
                        }
                        error => error.into(),
                    })?;
                let bytes = name.len() as u64 + 1;
                self.name_bytes = self
                    .name_bytes
                    .checked_add(bytes)
                    .ok_or(RawFileError::SizeOverflow)?;
                raw_file_error::check(
                    RawFileResource::TotalNameBytes,
                    self.name_bytes,
                    self.budget.max_total_name_bytes,
                )?;
                self.advance(bytes)?;
                self.registry.add_name(offset, name)?;
                name
            }
            ZonePointer::Offset(address) => self.registry.name(address, self.header)?,
            ZonePointer::Null | ZonePointer::InlineWithAlias => {
                return Err(RawFileError::InvalidNamePointer);
            }
        };
        raw_file_error::check(
            RawFileResource::NameBytes,
            name.len() as u64 + 1,
            self.budget.max_name_bytes,
        )?;
        Ok(name)
    }

    fn read_buffer(
        &mut self,
        length: u32,
        present: bool,
    ) -> Result<Option<&'payload [u8]>, RawFileError> {
        if !present {
            if length != 0 {
                return Err(RawFileError::MissingBuffer);
            }
            return Ok(None);
        }
        let bytes = u64::from(length) + 1;
        raw_file_error::check(
            RawFileResource::BufferBytes,
            bytes,
            self.budget.max_buffer_bytes,
        )?;
        self.buffer_bytes = self
            .buffer_bytes
            .checked_add(bytes)
            .ok_or(RawFileError::SizeOverflow)?;
        raw_file_error::check(
            RawFileResource::TotalBufferBytes,
            self.buffer_bytes,
            self.budget.max_total_buffer_bytes,
        )?;
        self.advance(bytes)?;
        let size = usize::try_from(bytes).map_err(|_| RawFileError::SizeOverflow)?;
        let buffer = self.cursor.take(size)?;
        if buffer.last() != Some(&0) {
            return Err(RawFileError::MissingTerminator);
        }
        Ok(Some(&buffer[..size - 1]))
    }

    fn advance(&mut self, bytes: u64) -> Result<(), RawFileError> {
        let end = self
            .virtual_position
            .checked_add(bytes)
            .ok_or(RawFileError::SizeOverflow)?;
        if end > self.header.block_bytes()[4] {
            return Err(RawFileError::BlockBounds);
        }
        self.virtual_position = end;
        Ok(())
    }
}
