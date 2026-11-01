// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::{KernelErrorKind, KernelValue};
use crate::legacy::LegacyVectorError;

pub(super) fn read(
    object: KernelValue,
    index: KernelValue,
) -> Result<KernelValue, KernelErrorKind> {
    let KernelValue::Vector(object) = object else {
        return Err(KernelErrorKind::ExpectedIndexable);
    };
    let KernelValue::Number(index) = index else {
        return Err(KernelErrorKind::Vector(
            LegacyVectorError::ExpectedIntegerIndex,
        ));
    };
    object
        .index(index)
        .map(KernelValue::Number)
        .map_err(KernelErrorKind::Vector)
}
