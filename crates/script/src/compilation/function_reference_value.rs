// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::compilation::function_builder::FunctionBuilder;
use crate::compilation::{FunctionCompileError, FunctionCompileErrorKind};
use crate::compilation::{function_call_target, function_compile_error};
use crate::execution::KernelInstruction;
use crate::expression::LegacyFunctionNameSyntax;
use crate::syntax::LegacySyntaxNode;

pub(super) fn lower(
    builder: &mut FunctionBuilder<'_, '_>,
    node: LegacySyntaxNode<'_, '_>,
) -> Result<(), FunctionCompileError> {
    let callee = LegacyFunctionNameSyntax::from_node(node).map_err(|_| {
        function_compile_error::at(node.span(), FunctionCompileErrorKind::InvalidStructure)
    })?;
    let function = function_call_target::resolve(builder, callee)?;
    builder.emit(KernelInstruction::FunctionReference(function), node.span())
}
