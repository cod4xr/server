// SPDX-License-Identifier: AGPL-3.0-or-later
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::heap::objects::object_work::ObjectWork;
use crate::heap::{ArrayKey, ObjectHeapErrorKind};

/// Borrowed lookup/copy plan. Array keys have a private, lossless tagged encoding;
/// struct keys remain exact bytes. Parent classification prevents cross-use.
#[derive(Clone, Copy)]
pub(super) enum FieldKey<'a> {
    Bytes(&'a [u8]),
    Array(ArrayKey<'a>),
}

impl FieldKey<'_> {
    pub(super) fn len(self) -> Result<usize, ObjectHeapErrorKind> {
        match self {
            Self::Bytes(bytes) => Ok(bytes.len()),
            Self::Array(ArrayKey::Index(_)) => Ok(9),
            Self::Array(ArrayKey::Bytes(bytes)) => bytes
                .len()
                .checked_add(1)
                .ok_or(ObjectHeapErrorKind::SizeOverflow),
        }
    }

    pub(super) fn hash(self, owner: usize) -> u64 {
        let mut hasher = DefaultHasher::new();
        owner.hash(&mut hasher);
        match self {
            Self::Bytes(bytes) => bytes.hash(&mut hasher),
            Self::Array(ArrayKey::Index(index)) => {
                0_u8.hash(&mut hasher);
                index.hash(&mut hasher);
            }
            Self::Array(ArrayKey::Bytes(bytes)) => {
                1_u8.hash(&mut hasher);
                bytes.hash(&mut hasher);
            }
        }
        hasher.finish()
    }

    pub(super) fn matches(self, stored: &[u8]) -> bool {
        match self {
            Self::Bytes(bytes) => stored == bytes,
            Self::Array(ArrayKey::Index(index)) => {
                stored.strip_prefix(&[0]) == Some(index.to_le_bytes().as_slice())
            }
            Self::Array(ArrayKey::Bytes(bytes)) => stored.strip_prefix(&[1]) == Some(bytes),
        }
    }

    pub(super) fn copy(self, work: &mut ObjectWork) -> Result<Vec<u8>, ObjectHeapErrorKind> {
        let count = self.len()?;
        work.charge(count as u64)?;
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(count)
            .map_err(|_| ObjectHeapErrorKind::AllocationFailed)?;
        match self {
            Self::Bytes(key) => bytes.extend_from_slice(key),
            Self::Array(ArrayKey::Index(index)) => {
                bytes.push(0);
                bytes.extend_from_slice(&index.to_le_bytes());
            }
            Self::Array(ArrayKey::Bytes(key)) => {
                bytes.push(1);
                bytes.extend_from_slice(key);
            }
        }
        Ok(bytes)
    }

    pub(super) fn decode_array(stored: &[u8]) -> Result<ArrayKey<'_>, ObjectHeapErrorKind> {
        match stored.split_first() {
            Some((0, rest)) => {
                let word: [u8; 8] = rest
                    .try_into()
                    .map_err(|_| ObjectHeapErrorKind::InvalidState)?;
                Ok(ArrayKey::Index(u64::from_le_bytes(word)))
            }
            Some((1, bytes)) => Ok(ArrayKey::Bytes(bytes)),
            _ => Err(ObjectHeapErrorKind::InvalidState),
        }
    }
}
