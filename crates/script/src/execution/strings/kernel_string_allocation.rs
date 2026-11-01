// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::strings::kernel_heap_collection;
use crate::execution::{KernelErrorKind, KernelHeapContext, KernelResource, KernelValue};
use crate::heap::StringId;

#[derive(Clone, Copy)]
pub(super) enum KernelStringInput<'a> {
    Literal(&'a [u8]),
    Pair(StringId, StringId),
}

impl KernelHeapContext<'_> {
    pub(super) fn concatenate(
        &mut self,
        left: StringId,
        right: StringId,
        roots: impl Iterator<Item = KernelValue> + Clone,
    ) -> Result<StringId, KernelErrorKind> {
        self.allocate(KernelStringInput::Pair(left, right), roots)
    }

    pub(super) fn allocate(
        &mut self,
        input: KernelStringInput<'_>,
        roots: impl Iterator<Item = KernelValue> + Clone,
    ) -> Result<StringId, KernelErrorKind> {
        match self.attempt(input) {
            Ok(id) => return Ok(id),
            Err(KernelErrorKind::BudgetExceeded {
                resource:
                    KernelResource::StringValues
                    | KernelResource::StringPayloadBytes
                    | KernelResource::StringSlots
                    | KernelResource::StringMetadataBytes,
                ..
            }) => {}
            Err(kind) => return Err(kind),
        }
        kernel_heap_collection::collect(self, roots)?;
        self.attempt(input)
    }

    fn attempt(&mut self, input: KernelStringInput<'_>) -> Result<StringId, KernelErrorKind> {
        let budget = self.remaining(0)?;
        let result = match input {
            KernelStringInput::Literal(bytes) => self.heap.allocate(bytes, budget),
            KernelStringInput::Pair(left, right) => self.heap.concatenate(left, right, budget),
        };
        match result {
            Ok(result) => {
                self.record(result.usage, 0)?;
                Ok(result.id)
            }
            Err(error) => Err(self.record_error(error, 0)?),
        }
    }
}
