// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::heap::kernel_heap_error;
use crate::execution::{KernelErrorKind, KernelValue};
use crate::heap::{ArrayKey, LegacyArrayKeyError, StringHeap};

pub(super) fn legacy_index(
    value: KernelValue,
    strings: &StringHeap,
) -> Result<ArrayKey<'_>, KernelErrorKind> {
    ArrayKey::from_legacy_value(value, strings).map_err(|error| match error {
        LegacyArrayKeyError::ExpectedIndex => KernelErrorKind::ExpectedArrayIndex,
        LegacyArrayKeyError::IndexOutOfRange => KernelErrorKind::ArrayIndexOutOfRange,
        LegacyArrayKeyError::StringHeap(error) => kernel_heap_error::map(error),
    })
}
