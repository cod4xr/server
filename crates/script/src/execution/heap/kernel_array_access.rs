// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::heap::kernel_array_keys;
use crate::execution::heap::kernel_object_error;
use crate::execution::{KernelErrorKind, KernelHeapContext, KernelValue};
use crate::heap::{ArrayId, ArrayValueId};
use crate::legacy::LegacyNumber;

impl KernelHeapContext<'_> {
    // Fields cannot contain Array values. Releases do not allocate/collect/yield,
    // so a checked result reaches its frame before an unowned receiver's payload
    // can be reclaimed. Later collection must include that frame's result roots.
    // No allocation, COW detach, field creation or independent owner originates here.
    pub(super) fn array_index(
        &mut self,
        value: ArrayValueId,
        index: KernelValue,
    ) -> Result<KernelValue, KernelErrorKind> {
        let data = self.array_snapshot(value)?;
        self.charge(1)?;
        let key = kernel_array_keys::legacy_index(index, self.heap)?;
        let budget = self.remaining(0)?;
        let result = self
            .objects
            .as_deref()
            .ok_or(KernelErrorKind::ObjectExecutionUnsupported)?
            .array_get(data, key, self.heap, budget);
        match result {
            Ok(result) => {
                self.record(result.usage, 0)?;
                Ok(result.value.unwrap_or(KernelValue::Undefined))
            }
            Err(error) => Err(kernel_object_error::record(self, error, 0)?),
        }
    }

    pub(super) fn array_size(
        &mut self,
        value: ArrayValueId,
    ) -> Result<KernelValue, KernelErrorKind> {
        let data = self.array_snapshot(value)?;
        let budget = self.remaining(0)?;
        let result = self
            .objects
            .as_deref()
            .ok_or(KernelErrorKind::ObjectExecutionUnsupported)?
            .array_len(data, budget);
        match result {
            Ok(result) => {
                self.record(result.usage, 0)?;
                let length =
                    i32::try_from(result.value).map_err(|_| KernelErrorKind::SizeOverflow)?;
                Ok(KernelValue::Number(LegacyNumber::Integer(length)))
            }
            Err(error) => Err(kernel_object_error::record(self, error, 0)?),
        }
    }

    fn array_snapshot(&mut self, value: ArrayValueId) -> Result<ArrayId, KernelErrorKind> {
        self.charge(1)?;
        let budget = self.remaining(0)?;
        let result = self
            .objects
            .as_deref()
            .ok_or(KernelErrorKind::ObjectExecutionUnsupported)?
            .array_data(value, budget);
        match result {
            Ok(result) => {
                self.record(result.usage, 0)?;
                Ok(result.value)
            }
            Err(error) => Err(kernel_object_error::record(self, error, 0)?),
        }
    }
}
