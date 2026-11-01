// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::binding::LegacyIdentifier;
use crate::compilation::function_builder::FunctionBuilder;
use crate::compilation::function_compile_error;
use crate::compilation::function_local_bindings::{FunctionLocalDeclaration, FunctionLocalSlot};
use crate::compilation::{FunctionCompileError, FunctionCompileErrorKind};
use crate::execution::KernelInstruction;
use crate::lexical::LegacyLexeme;

pub(super) fn declare<'source>(
    builder: &mut FunctionBuilder<'_, 'source>,
    token: LegacyLexeme<'source>,
    declaration: FunctionLocalDeclaration,
) -> Result<(LegacyIdentifier<'source>, usize), FunctionCompileError> {
    let name = identifier(builder, token)?;
    let error = |kind| function_compile_error::at(token.span(), kind);
    let table = &builder
        .locals
        .as_ref()
        .ok_or_else(|| error(FunctionCompileErrorKind::InvalidLocalBinding))?
        .names;
    if let Some(slot) = table.get(&name) {
        return match declaration {
            FunctionLocalDeclaration::Parameter => {
                Err(error(FunctionCompileErrorKind::DuplicateParameter))
            }
            FunctionLocalDeclaration::Assignment => Ok((name, slot.index)),
        };
    }
    let index = table.len();
    let count = index
        .checked_add(1)
        .ok_or_else(|| error(FunctionCompileErrorKind::SizeOverflow))?;
    // Logical key/value payload only: hash control/capacity/allocator overhead is
    // excluded explicitly, not presented as an aggregate heap/RSS bound.
    let bytes = (count as u64)
        .checked_mul(size_of::<(LegacyIdentifier<'_>, FunctionLocalSlot)>() as u64)
        .ok_or_else(|| error(FunctionCompileErrorKind::SizeOverflow))?;
    builder.admit_binding(count as u64, bytes, token.span())?;
    let table = &mut builder
        .locals
        .as_mut()
        .ok_or_else(|| error(FunctionCompileErrorKind::InvalidLocalBinding))?
        .names;
    table
        .try_reserve(1)
        .map_err(|_| error(FunctionCompileErrorKind::AllocationFailed))?;
    let initialized = matches!(declaration, FunctionLocalDeclaration::Parameter);
    if table
        .insert(name, FunctionLocalSlot { index, initialized })
        .is_some()
    {
        return Err(error(FunctionCompileErrorKind::InvalidLocalBinding));
    }
    Ok((name, index))
}

pub(super) fn read<'source>(
    builder: &mut FunctionBuilder<'_, 'source>,
    token: LegacyLexeme<'source>,
) -> Result<(), FunctionCompileError> {
    let (_, slot) = lookup(builder, token)?;
    builder.emit(KernelInstruction::LoadLocal(slot), token.span())
}

pub(super) fn lookup<'source>(
    builder: &mut FunctionBuilder<'_, 'source>,
    token: LegacyLexeme<'source>,
) -> Result<(LegacyIdentifier<'source>, usize), FunctionCompileError> {
    let name = identifier(builder, token)?;
    let slot = builder
        .locals
        .as_ref()
        .and_then(|locals| locals.names.get(&name))
        .filter(|slot| slot.initialized)
        .copied()
        .ok_or_else(|| {
            function_compile_error::at(token.span(), FunctionCompileErrorKind::UninitializedLocal)
        })?;
    Ok((name, slot.index))
}

pub(super) fn initialize<'source>(
    builder: &mut FunctionBuilder<'_, 'source>,
    name: LegacyIdentifier<'source>,
    index: usize,
) -> Result<(), FunctionCompileError> {
    builder
        .ledger
        .identifier(name.as_bytes().len() as u64, name.lexeme().span())?;
    let slot = builder
        .locals
        .as_mut()
        .and_then(|locals| locals.names.get_mut(&name))
        .filter(|slot| slot.index == index)
        .ok_or_else(|| {
            function_compile_error::at(
                name.lexeme().span(),
                FunctionCompileErrorKind::InvalidLocalBinding,
            )
        })?;
    slot.initialized = true;
    Ok(())
}

fn identifier<'source>(
    builder: &mut FunctionBuilder<'_, '_>,
    token: LegacyLexeme<'source>,
) -> Result<LegacyIdentifier<'source>, FunctionCompileError> {
    builder
        .ledger
        .identifier(token.as_bytes().len() as u64, token.span())?;
    LegacyIdentifier::from_lexeme(token).map_err(|error| {
        function_compile_error::at(token.span(), FunctionCompileErrorKind::Identifier(error))
    })
}
