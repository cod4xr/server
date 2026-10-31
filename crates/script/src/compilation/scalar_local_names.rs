// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::binding::LegacyIdentifier;
use crate::compilation::scalar_builder::ScalarBuilder;
use crate::compilation::scalar_compile_error;
use crate::compilation::scalar_local_bindings::{ScalarLocalDeclaration, ScalarLocalSlot};
use crate::compilation::{ScalarCompileError, ScalarCompileErrorKind};
use crate::execution::KernelInstruction;
use crate::expression::{LegacyReferenceKind, LegacyReferenceSyntax};
use crate::lexical::LegacyLexeme;
use crate::syntax::LegacySyntaxNode;

pub(super) fn declare<'source>(
    builder: &mut ScalarBuilder<'_, 'source>,
    token: LegacyLexeme<'source>,
    declaration: ScalarLocalDeclaration,
) -> Result<(LegacyIdentifier<'source>, usize), ScalarCompileError> {
    let name = identifier(builder, token)?;
    let error = |kind| scalar_compile_error::at(token.span(), kind);
    let table = &builder
        .locals
        .as_ref()
        .ok_or_else(|| error(ScalarCompileErrorKind::InvalidLocalBinding))?
        .names;
    if let Some(slot) = table.get(&name) {
        return match declaration {
            ScalarLocalDeclaration::Parameter => {
                Err(error(ScalarCompileErrorKind::DuplicateParameter))
            }
            ScalarLocalDeclaration::Assignment => Ok((name, slot.index)),
        };
    }
    let index = table.len();
    let count = index
        .checked_add(1)
        .ok_or_else(|| error(ScalarCompileErrorKind::SizeOverflow))?;
    // Logical key/value payload only: hash control/capacity/allocator overhead is
    // excluded explicitly, not presented as an aggregate heap/RSS bound.
    let bytes = (count as u64)
        .checked_mul(size_of::<(LegacyIdentifier<'_>, ScalarLocalSlot)>() as u64)
        .ok_or_else(|| error(ScalarCompileErrorKind::SizeOverflow))?;
    builder.admit_binding(count as u64, bytes, token.span())?;
    let table = &mut builder
        .locals
        .as_mut()
        .ok_or_else(|| error(ScalarCompileErrorKind::InvalidLocalBinding))?
        .names;
    table
        .try_reserve(1)
        .map_err(|_| error(ScalarCompileErrorKind::AllocationFailed))?;
    let initialized = matches!(declaration, ScalarLocalDeclaration::Parameter);
    if table
        .insert(name, ScalarLocalSlot { index, initialized })
        .is_some()
    {
        return Err(error(ScalarCompileErrorKind::InvalidLocalBinding));
    }
    Ok((name, index))
}

pub(super) fn read<'tree, 'source>(
    builder: &mut ScalarBuilder<'tree, 'source>,
    syntax: LegacySyntaxNode<'tree, 'source>,
) -> Result<(), ScalarCompileError> {
    let reference = LegacyReferenceSyntax::from_node(syntax).map_err(|_| {
        scalar_compile_error::at(syntax.span(), ScalarCompileErrorKind::InvalidStructure)
    })?;
    let LegacyReferenceKind::Name { name: token } = reference.kind() else {
        return Err(scalar_compile_error::at(
            syntax.span(),
            ScalarCompileErrorKind::UnsupportedPrimary,
        ));
    };
    let (_, slot) = lookup(builder, token)?;
    builder.emit(KernelInstruction::LoadLocal(slot), token.span())
}

pub(super) fn lookup<'source>(
    builder: &mut ScalarBuilder<'_, 'source>,
    token: LegacyLexeme<'source>,
) -> Result<(LegacyIdentifier<'source>, usize), ScalarCompileError> {
    let name = identifier(builder, token)?;
    let slot = builder
        .locals
        .as_ref()
        .and_then(|locals| locals.names.get(&name))
        .filter(|slot| slot.initialized)
        .copied()
        .ok_or_else(|| {
            scalar_compile_error::at(token.span(), ScalarCompileErrorKind::UninitializedLocal)
        })?;
    Ok((name, slot.index))
}

pub(super) fn initialize<'source>(
    builder: &mut ScalarBuilder<'_, 'source>,
    name: LegacyIdentifier<'source>,
    index: usize,
) -> Result<(), ScalarCompileError> {
    builder
        .ledger
        .identifier(name.as_bytes().len() as u64, name.lexeme().span())?;
    let slot = builder
        .locals
        .as_mut()
        .and_then(|locals| locals.names.get_mut(&name))
        .filter(|slot| slot.index == index)
        .ok_or_else(|| {
            scalar_compile_error::at(
                name.lexeme().span(),
                ScalarCompileErrorKind::InvalidLocalBinding,
            )
        })?;
    slot.initialized = true;
    Ok(())
}

fn identifier<'source>(
    builder: &mut ScalarBuilder<'_, '_>,
    token: LegacyLexeme<'source>,
) -> Result<LegacyIdentifier<'source>, ScalarCompileError> {
    builder
        .ledger
        .identifier(token.as_bytes().len() as u64, token.span())?;
    LegacyIdentifier::from_lexeme(token).map_err(|error| {
        scalar_compile_error::at(token.span(), ScalarCompileErrorKind::Identifier(error))
    })
}
