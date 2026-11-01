// SPDX-License-Identifier: AGPL-3.0-or-later
use super::switch_error;
use crate::compilation::FunctionCompileError;
use crate::compilation::function_builder::FunctionBuilder;
use crate::execution::{
    KernelBranchCondition, KernelInstruction as Op, KernelSwitchComparison, KernelSwitchKey,
    KernelValue,
};
use crate::legacy::{LegacyComparison, LegacyNumber};
use std::ops::Range;

pub(super) const LEGACY_WIDTH: usize = 11;
pub(super) const BOUND_WIDTH: usize = 9;

pub(super) fn emit(
    builder: &mut FunctionBuilder<'_, '_>,
    key: KernelSwitchKey,
    pc: usize,
    target: usize,
    children: (usize, usize),
    span: Range<u64>,
) -> Result<(), FunctionCompileError> {
    let (left, right) = children;
    if builder.mode.bound() {
        for instruction in [
            Op::Duplicate,
            Op::SwitchCompare {
                key,
                comparison: KernelSwitchComparison::Equal,
            },
            Op::Branch {
                condition: KernelBranchCondition::IfFalse,
                target: pc + 5,
            },
            Op::Discard,
            Op::Jump(target),
            Op::Duplicate,
            Op::SwitchCompare {
                key,
                comparison: KernelSwitchComparison::Less,
            },
            Op::Branch {
                condition: KernelBranchCondition::IfTrue,
                target: left,
            },
            Op::Jump(right),
        ] {
            builder.emit(instruction, span.clone())?;
        }
    } else {
        let KernelSwitchKey::Integer(key) = key else {
            return Err(switch_error::invalid(span));
        };
        let constant = Op::Constant(KernelValue::Number(LegacyNumber::Integer(key)));
        for instruction in [
            Op::Duplicate,
            constant,
            Op::Compare(LegacyComparison::Equal),
            Op::Branch {
                condition: KernelBranchCondition::IfFalse,
                target: pc + 6,
            },
            Op::Discard,
            Op::Jump(target),
            Op::Duplicate,
            constant,
            Op::Compare(LegacyComparison::Less),
            Op::Branch {
                condition: KernelBranchCondition::IfTrue,
                target: left,
            },
            Op::Jump(right),
        ] {
            builder.emit(instruction, span.clone())?;
        }
    }
    Ok(())
}
