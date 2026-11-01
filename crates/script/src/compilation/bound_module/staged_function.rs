// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::compilation::bound_module::CompiledModuleFunction;
use crate::compilation::function_builder::FunctionBuilder;
use crate::compilation::function_lowering_mode::FunctionLoweringMode;
use crate::compilation::primitive_function_compiler;
use crate::compilation::{FunctionCompileError, FunctionLoweringBudget};
use crate::declaration::LegacyFunctionSyntax;
use crate::execution::{KernelFunctionLayout, KernelInstruction};

pub(super) struct StagedFunctionCode {
    pub(super) string_literals: Vec<Vec<u8>>,
    pub(super) field_names: Vec<Vec<u8>>,
    pub(super) switch_labels: Vec<Vec<u8>>,
    pub(super) instructions: Vec<KernelInstruction>,
    pub(super) layout: KernelFunctionLayout,
}

pub(super) fn lower<'tree, 'source>(
    function: &LegacyFunctionSyntax<'tree, 'source>,
    declaration: usize,
    mode: FunctionLoweringMode<'tree, 'source>,
    budget: FunctionLoweringBudget,
) -> Result<(StagedFunctionCode, CompiledModuleFunction), FunctionCompileError> {
    let mut builder = FunctionBuilder::with_mode(budget, mode);
    let result = primitive_function_compiler::lower(&mut builder, function).and_then(|layout| {
        let (instructions, spans) = builder.resolve(function.body().span())?;
        Ok((
            StagedFunctionCode {
                string_literals: std::mem::take(&mut builder.string_literals),
                field_names: std::mem::take(&mut builder.field_names),
                switch_labels: std::mem::take(&mut builder.switch_labels),
                instructions,
                layout,
            },
            CompiledModuleFunction {
                declaration,
                spans,
                usage: builder.ledger.usage,
            },
        ))
    });
    result.map_err(|mut error| {
        error.work_steps = builder.ledger.usage.compiler_steps;
        error.peak_metadata_bytes = builder.ledger.usage.peak_metadata_bytes;
        error
    })
}
