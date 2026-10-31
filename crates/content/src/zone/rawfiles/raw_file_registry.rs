// SPDX-License-Identifier: AGPL-3.0-or-later
use std::collections::HashMap;

use crate::fastfile::FastFileHeader;
use crate::zone::ZoneAddress;
use crate::zone::rawfiles::raw_file_error;
use crate::zone::rawfiles::{RawFile, RawFileError, RawFileResource};

struct NameRegion<'payload> {
    offset: u64,
    bytes: &'payload [u8],
}

pub(super) struct RawFileRegistry<'payload> {
    aliases: HashMap<u64, Option<RawFile<'payload>>>,
    names: Vec<NameRegion<'payload>>,
    metadata: u64,
    maximum: u64,
}

impl<'payload> RawFileRegistry<'payload> {
    pub(super) fn metadata_bytes(&self) -> u64 {
        self.metadata
    }

    pub(super) fn new(entries: usize, maximum: u64) -> Result<Self, RawFileError> {
        let metadata = entries
            .checked_mul(size_of::<Option<RawFile<'_>>>())
            .ok_or(RawFileError::SizeOverflow)? as u64;
        raw_file_error::check(RawFileResource::MetadataBytes, metadata, maximum)?;
        Ok(Self {
            aliases: HashMap::new(),
            names: Vec::new(),
            metadata,
            maximum,
        })
    }

    fn admit(&mut self, bytes: usize) -> Result<(), RawFileError> {
        let size = self
            .metadata
            .checked_add(bytes as u64)
            .ok_or(RawFileError::SizeOverflow)?;
        raw_file_error::check(RawFileResource::MetadataBytes, size, self.maximum)?;
        self.metadata = size;
        Ok(())
    }

    pub(super) fn bind(
        &mut self,
        offset: u64,
        target: Option<RawFile<'payload>>,
    ) -> Result<(), RawFileError> {
        if !self.aliases.contains_key(&offset) {
            self.admit(size_of::<(u64, Option<RawFile<'_>>)>())?;
            self.aliases
                .try_reserve(1)
                .map_err(|_| RawFileError::AllocationFailed)?;
        }
        let _ = self.aliases.insert(offset, target);
        Ok(())
    }

    pub(super) fn alias(
        &self,
        address: ZoneAddress,
        header: FastFileHeader,
    ) -> Result<Option<RawFile<'payload>>, RawFileError> {
        address.validate_span(header, 4)?;
        if address.block() != 4 {
            return Err(RawFileError::UnknownAlias);
        }
        self.aliases
            .get(&address.offset())
            .copied()
            .ok_or(RawFileError::UnknownAlias)
    }

    pub(super) fn add_name(
        &mut self,
        offset: u64,
        bytes: &'payload [u8],
    ) -> Result<(), RawFileError> {
        self.admit(size_of::<NameRegion<'_>>())?;
        self.names
            .try_reserve(1)
            .map_err(|_| RawFileError::AllocationFailed)?;
        self.names.push(NameRegion { offset, bytes });
        Ok(())
    }

    pub(super) fn name(
        &self,
        address: ZoneAddress,
        header: FastFileHeader,
    ) -> Result<&'payload [u8], RawFileError> {
        address.validate_span(header, 1)?;
        if address.block() != 4 {
            return Err(RawFileError::UnknownName);
        }
        let index = self
            .names
            .partition_point(|region| region.offset <= address.offset());
        let region = index
            .checked_sub(1)
            .and_then(|index| self.names.get(index))
            .ok_or(RawFileError::UnknownName)?;
        let delta = usize::try_from(address.offset() - region.offset)
            .map_err(|_| RawFileError::SizeOverflow)?;
        region.bytes.get(delta..).ok_or(RawFileError::UnknownName)
    }
}
