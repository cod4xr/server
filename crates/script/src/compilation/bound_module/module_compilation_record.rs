// SPDX-License-Identifier: AGPL-3.0-or-later
use super::module_compilation_ledger::{ModuleCompilationLedger, add};
use crate::compilation::{
    FunctionCompilationUsage, FunctionCompileError, ModuleCompilationResource,
};
use std::ops::Range;

pub(super) fn record(
    ledger: &mut ModuleCompilationLedger,
    usage: FunctionCompilationUsage,
    span: Range<u64>,
) -> Result<(), FunctionCompileError> {
    let peak = add(
        ledger.prefix(span.clone())?,
        usage.peak_metadata_bytes,
        span.clone(),
    )?;
    ledger.metadata(peak, span.clone())?;
    ledger.charge(usage.compiler_steps, span.clone())?;
    let peak_instructions = add(
        ledger.usage.instructions,
        usage.peak_instructions,
        span.clone(),
    )?;
    ledger.check(
        ModuleCompilationResource::Instructions,
        peak_instructions,
        ledger.budget.max_instructions,
        span.clone(),
    )?;
    ledger.usage.peak_instructions = ledger.usage.peak_instructions.max(peak_instructions);
    let instructions = add(ledger.usage.instructions, usage.instructions, span.clone())?;
    ledger.check(
        ModuleCompilationResource::Instructions,
        instructions,
        ledger.budget.max_instructions,
        span.clone(),
    )?;
    ledger.usage.instructions = instructions;
    ledger.usage.string_literals = add(
        ledger.usage.string_literals,
        usage.string_literals,
        span.clone(),
    )?;
    ledger.usage.staged_string_bytes = add(
        ledger.usage.staged_string_bytes,
        usage.retained_string_bytes,
        span.clone(),
    )?;
    ledger.usage.field_names = add(ledger.usage.field_names, usage.field_names, span.clone())?;
    ledger.usage.staged_field_bytes = add(
        ledger.usage.staged_field_bytes,
        usage.retained_field_bytes,
        span.clone(),
    )?;
    ledger.usage.switch_labels = add(
        ledger.usage.switch_labels,
        usage.switch_labels,
        span.clone(),
    )?;
    ledger.usage.staged_switch_bytes = add(
        ledger.usage.staged_switch_bytes,
        usage.retained_switch_bytes,
        span,
    )?;
    Ok(())
}
