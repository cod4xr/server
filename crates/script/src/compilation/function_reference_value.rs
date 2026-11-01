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
    if crate::compilation::function_developer::discarded(builder).is_some() {
        crate::compilation::function_developer::validate_name(builder, callee)?;
        return crate::compilation::function_developer::emit_validation(builder, node.span());
    }
    let function = function_call_target::resolve(builder, callee)?;
    builder.emit(KernelInstruction::FunctionReference(function), node.span())
}
