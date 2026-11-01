// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::heap::kernel_object_validation;
use crate::execution::{KernelErrorKind, KernelHeapContext, KernelValue};
use crate::legacy::{LegacyComparison, LegacyNumber};

pub(super) fn compare(
    context: Option<&mut KernelHeapContext<'_>>,
    comparison: LegacyComparison,
    left: KernelValue,
    right: KernelValue,
) -> Result<KernelValue, KernelErrorKind> {
    let (KernelValue::Object(left), KernelValue::Object(right)) = (left, right) else {
        return Err(KernelErrorKind::UnmatchingTypes);
    };
    if !matches!(
        comparison,
        LegacyComparison::Equal | LegacyComparison::NotEqual
    ) {
        return Err(KernelErrorKind::UnmatchingTypes);
    }
    let context = context.ok_or(KernelErrorKind::ObjectExecutionUnsupported)?;
    context.charge(2)?;
    kernel_object_validation::reference(context, left)?;
    kernel_object_validation::reference(context, right)?;
    // Checked owner/slot/generation identity is the language rule here. Do not
    // use KernelValue's data equality or visit fields/graph contents.
    let equal = left == right;
    let result = if comparison == LegacyComparison::Equal {
        equal
    } else {
        !equal
    };
    Ok(KernelValue::Number(LegacyNumber::Integer(i32::from(
        result,
    ))))
}
