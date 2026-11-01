// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::heap::kernel_heap_error;
use crate::execution::heap::kernel_string_allocation::KernelStringInput;
use crate::execution::{KernelErrorKind, KernelHeapContext, KernelValue};
use crate::heap::StringId;
use crate::legacy::LegacyNumber;

impl KernelHeapContext<'_> {
    pub(super) fn index(
        &mut self,
        object: StringId,
        index: KernelValue,
        roots: impl Iterator<Item = KernelValue> + Clone,
    ) -> Result<KernelValue, KernelErrorKind> {
        let KernelValue::Number(LegacyNumber::Integer(index)) = index else {
            return Err(KernelErrorKind::ExpectedStringIndex);
        };
        self.charge(1)?;
        let length = self.heap.get(object).map_err(kernel_heap_error::map)?.len();
        let index = usize::try_from(index)
            .ok()
            .filter(|&index| index < length)
            .ok_or(KernelErrorKind::StringIndexOutOfRange)?;
        let visits = index.checked_add(1).ok_or(KernelErrorKind::SizeOverflow)?;
        self.charge(visits as u64)?;
        let bytes = self.heap.get(object).map_err(kernel_heap_error::map)?;
        if bytes[..visits].contains(&0) {
            return Err(KernelErrorKind::StringIndexOutOfRange);
        }
        let byte = [bytes[index]];
        self.allocate(KernelStringInput::Literal(&byte), roots)
            .map(KernelValue::String)
    }

    pub(super) fn size(&mut self, object: StringId) -> Result<KernelValue, KernelErrorKind> {
        self.charge(1)?;
        let length = self.heap.get(object).map_err(kernel_heap_error::map)?.len();
        // Admit the complete possible scan before inspecting bytes. This is a
        // conservative logical bound, including unused bytes after an early NUL.
        self.charge(length as u64)?;
        let bytes = self.heap.get(object).map_err(kernel_heap_error::map)?;
        let length = bytes.iter().position(|&byte| byte == 0).unwrap_or(length);
        let length = i32::try_from(length).map_err(|_| KernelErrorKind::SizeOverflow)?;
        Ok(KernelValue::Number(LegacyNumber::Integer(length)))
    }
}
