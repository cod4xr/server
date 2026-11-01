// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::heap::kernel_heap_error;
use crate::execution::{KernelErrorKind, KernelHeapContext};
use crate::heap::StringId;
use std::cmp::Ordering;

impl KernelHeapContext<'_> {
    pub(super) fn require_switch_string(&mut self, id: StringId) -> Result<(), KernelErrorKind> {
        self.charge(1)?;
        self.heap.get(id).map_err(kernel_heap_error::map)?;
        Ok(())
    }

    pub(super) fn compare_switch_string(
        &mut self,
        id: StringId,
        label: &[u8],
    ) -> Result<Ordering, KernelErrorKind> {
        self.charge(1)?;
        let length = self.heap.get(id).map_err(kernel_heap_error::map)?.len();
        // Admit the complete possible prefix scan; fixed length/kind comparisons
        // use the instruction visit. No allocation, collection or interning occurs.
        self.charge(length.min(label.len()) as u64)?;
        let bytes = self.heap.get(id).map_err(kernel_heap_error::map)?;
        Ok(bytes.cmp(label))
    }
}
