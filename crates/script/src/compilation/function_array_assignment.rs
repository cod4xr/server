// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::compilation::function_builder::FunctionBuilder;
use crate::compilation::function_task::FunctionTask;
use crate::compilation::{
    FunctionCompileError, FunctionCompileErrorKind, function_body, function_compile_error,
    function_literal, function_local_names,
};
use crate::execution::{KernelInstruction, KernelValue};
use crate::expression::{
    LegacyLiteralPolarity, LegacyPrimaryKind, LegacyPrimarySyntax, LegacyReferenceKind,
    LegacyReferenceSyntax,
};
use crate::lexical::LegacyLexemeKind;
use crate::syntax::LegacySyntaxNode;

/// Selects an existing initialized local location, never a copied receiver value.
pub(super) fn assign<'tree, 'source>(
    builder: &mut FunctionBuilder<'tree, 'source>,
    target: LegacySyntaxNode<'tree, 'source>,
    value: LegacySyntaxNode<'tree, 'source>,
) -> Result<bool, FunctionCompileError> {
    if !builder.mode.bound() {
        return Ok(false);
    }
    let error = || {
        function_compile_error::at(
            target.span(),
            FunctionCompileErrorKind::UnsupportedLocalTarget,
        )
    };
    let reference = LegacyReferenceSyntax::from_node(target).map_err(|_| error())?;
    let LegacyReferenceKind::Index { object, index } = reference.kind() else {
        return Ok(false);
    };
    let receiver = LegacyReferenceSyntax::from_node(object).map_err(|_| error())?;
    let LegacyReferenceKind::Name { name } = receiver.kind() else {
        return Err(error());
    };
    let (_, slot) = function_local_names::lookup(builder, name)?;
    let clear = literal_clear(builder, value)?;
    let span = function_body::child(target, 1, 4)?.span();
    builder.push_task(
        FunctionTask::Emit {
            instruction: if clear {
                KernelInstruction::ClearArrayLocal(slot)
            } else {
                KernelInstruction::StoreArrayLocal(slot)
            },
            span: span.clone(),
        },
        span,
    )?;
    // LIFO: normal RHS, key, then borrow the frame's owning location. A receiver
    // LoadLocal would manufacture an alias and incorrectly force COW.
    builder.push_task(FunctionTask::Visit(index), index.span())?;
    if !clear {
        builder.push_task(FunctionTask::Visit(value), value.span())?;
    }
    Ok(true)
}

fn literal_clear(
    builder: &mut FunctionBuilder<'_, '_>,
    value: LegacySyntaxNode<'_, '_>,
) -> Result<bool, FunctionCompileError> {
    let Ok(primary) = LegacyPrimarySyntax::from_node(value) else {
        return Ok(false);
    };
    let LegacyPrimaryKind::Literal { token, polarity } = primary.kind() else {
        return Ok(false);
    };
    if polarity != LegacyLiteralPolarity::Unchanged
        || !matches!(token.kind(), LegacyLexemeKind::Token(code) if code.as_u16() == 0x12a)
    {
        return Ok(false);
    }
    builder
        .ledger
        .charge(token.as_bytes().len() as u64, token.span())?;
    if function_literal::decode(token, polarity, builder.ledger.budget.literal)?
        != KernelValue::Undefined
    {
        return Err(function_compile_error::at(
            value.span(),
            FunctionCompileErrorKind::InvalidStructure,
        ));
    }
    // Grouped/called Undefined deliberately takes StoreArrayLocal instead: its
    // child selection has observable temporary quotas before the removal.
    Ok(true)
}
