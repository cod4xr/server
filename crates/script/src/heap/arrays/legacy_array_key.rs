// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::KernelValue;
use crate::heap::{ArrayKey, LegacyArrayKeyError, StringHeap};
use crate::legacy::LegacyNumber;

// Native IsValidArrayIndex's script domain, never a storage/modern key limit.
const LEGACY_INTEGER_KEY_END: u64 = 0x0080_0000;

impl<'a> ArrayKey<'a> {
    /// Converts one value under the strict legacy array-key profile.
    ///
    /// Integers must be in 0..8,388,608. Floats are not coerced. Plain strings
    /// borrow complete checked heap bytes without allocation/scanning/interner
    /// emulation; localized strings and all other current categories reject.
    /// This does not look up an array, detach COW state or model error cleanup.
    /// # Errors
    /// Rejects invalid categories, legacy index range and foreign/stale strings
    /// without mutation. Owned key/hash/work admission belongs to storage callers.
    pub fn from_legacy_value(
        value: KernelValue,
        strings: &'a StringHeap,
    ) -> Result<Self, LegacyArrayKeyError> {
        match value {
            KernelValue::Number(LegacyNumber::Integer(index)) => {
                let index =
                    u64::try_from(index).map_err(|_| LegacyArrayKeyError::IndexOutOfRange)?;
                if index >= LEGACY_INTEGER_KEY_END {
                    return Err(LegacyArrayKeyError::IndexOutOfRange);
                }
                Ok(Self::Index(index))
            }
            KernelValue::String(id) => strings
                .get(id)
                .map(Self::Bytes)
                .map_err(LegacyArrayKeyError::StringHeap),
            KernelValue::Undefined
            | KernelValue::Number(LegacyNumber::Float(_))
            | KernelValue::Vector(_)
            | KernelValue::Function(_)
            | KernelValue::LocalizedString(_)
            | KernelValue::Object(_) => Err(LegacyArrayKeyError::ExpectedIndex),
        }
    }
}
