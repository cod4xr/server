// SPDX-License-Identifier: AGPL-3.0-or-later
use std::collections::HashMap;
use std::collections::hash_map::Entry;

use crate::binding::symbol_ledger::SymbolLedger;
use crate::binding::{
    LegacyDeveloperMode, LegacyFunctionBinding, LegacyFunctionInclusion, LegacyFunctionVisibility,
    LegacyIdentifier, LegacyModuleSymbols, SymbolBudget, SymbolError, SymbolErrorKind,
};
use crate::declaration::{LegacyFunctionSyntax, LegacyModuleItem, LegacyModuleSyntax};

pub(super) fn bind<'module, 'tree, 'source>(
    module: &'module LegacyModuleSyntax<'tree, 'source>,
    mode: LegacyDeveloperMode,
    budget: SymbolBudget,
) -> Result<LegacyModuleSymbols<'module, 'tree, 'source>, SymbolError> {
    let mut ledger = preflight(module, mode, budget)?;
    let count = usize::try_from(ledger.usage.functions)
        .map_err(|_| failure(SymbolErrorKind::SizeOverflow, 0, None))?;
    ledger
        .work(ledger.usage.name_bytes)
        .map_err(|kind| failure(kind, 0, None))?;
    let mut functions: Vec<LegacyFunctionBinding<'module, 'tree, 'source>> = Vec::new();
    let mut names: HashMap<LegacyIdentifier<'source>, usize> = HashMap::new();
    functions
        .try_reserve_exact(count)
        .map_err(|_| failure(SymbolErrorKind::AllocationFailed, 0, None))?;
    names
        .try_reserve(
            usize::try_from(ledger.usage.active_functions)
                .map_err(|_| failure(SymbolErrorKind::SizeOverflow, 0, None))?,
        )
        .map_err(|_| failure(SymbolErrorKind::AllocationFailed, 0, None))?;
    let mut developer = None;
    let mut invalid_directive = None;
    for item in module.items() {
        let offset = item_offset(item);
        ledger.work(1).map_err(|kind| failure(kind, offset, None))?;
        match item {
            LegacyModuleItem::DeveloperBegin { offset } => {
                if developer.is_some() {
                    return Err(failure(
                        SymbolErrorKind::NestedDeveloperRegion,
                        *offset,
                        developer,
                    ));
                }
                developer = Some(*offset);
            }
            LegacyModuleItem::DeveloperEnd { offset } => {
                if developer.take().is_none() {
                    return Err(failure(
                        SymbolErrorKind::UnexpectedDeveloperEnd,
                        *offset,
                        None,
                    ));
                }
            }
            LegacyModuleItem::UsingAnimTree(_) => {
                if developer.is_some() {
                    invalid_directive.get_or_insert(failure(
                        SymbolErrorKind::AnimTreeInDeveloperRegion,
                        offset,
                        developer,
                    ));
                }
            }
            LegacyModuleItem::Function(declaration) => {
                let visibility = if developer.is_some() {
                    ledger.usage.developer_functions += 1;
                    LegacyFunctionVisibility::DeveloperOnly
                } else {
                    ledger.usage.ordinary_functions += 1;
                    LegacyFunctionVisibility::Ordinary
                };
                // Both counts stay <= the admitted, native-representable function count.
                let binding = index_function(
                    &mut names,
                    &functions,
                    declaration,
                    visibility,
                    mode.inclusion(visibility),
                )?;
                functions.push(binding);
            }
        }
    }
    if let Some(offset) = developer {
        return Err(failure(
            SymbolErrorKind::UnclosedDeveloperRegion,
            offset,
            None,
        ));
    }
    if let Some(error) = invalid_directive {
        return Err(error);
    }
    Ok(LegacyModuleSymbols {
        functions,
        names,
        usage: ledger.usage,
        mode,
    })
}

fn preflight(
    module: &LegacyModuleSyntax<'_, '_>,
    mode: LegacyDeveloperMode,
    budget: SymbolBudget,
) -> Result<SymbolLedger, SymbolError> {
    let mut ledger = SymbolLedger::new(budget);
    let mut developer = false;
    for item in module.items() {
        let offset = item_offset(item);
        ledger.work(1).map_err(|kind| failure(kind, offset, None))?;
        match item {
            LegacyModuleItem::DeveloperBegin { .. } => developer = true,
            LegacyModuleItem::DeveloperEnd { .. } => developer = false,
            LegacyModuleItem::Function(function) => {
                let visibility = if developer {
                    LegacyFunctionVisibility::DeveloperOnly
                } else {
                    LegacyFunctionVisibility::Ordinary
                };
                ledger
                    .function(function.name().as_bytes().len(), mode.inclusion(visibility))
                    .map_err(|kind| failure(kind, offset, None))?;
            }
            LegacyModuleItem::UsingAnimTree(_) => {}
        }
    }
    Ok(ledger)
}

fn index_function<'module, 'tree, 'source>(
    names: &mut HashMap<LegacyIdentifier<'source>, usize>,
    functions: &[LegacyFunctionBinding<'module, 'tree, 'source>],
    declaration: &'module LegacyFunctionSyntax<'tree, 'source>,
    visibility: LegacyFunctionVisibility,
    inclusion: LegacyFunctionInclusion,
) -> Result<LegacyFunctionBinding<'module, 'tree, 'source>, SymbolError> {
    let offset = declaration.name().span().start;
    let name = LegacyIdentifier::from_lexeme(declaration.name())
        .map_err(|error| failure(SymbolErrorKind::Identifier(error), offset, None))?;
    if inclusion == LegacyFunctionInclusion::Active {
        match names.entry(name) {
            Entry::Occupied(entry) => {
                let previous = functions
                    .get(*entry.get())
                    .ok_or_else(|| failure(SymbolErrorKind::InvalidState, offset, None))?;
                return Err(failure(
                    SymbolErrorKind::DuplicateFunction,
                    offset,
                    Some(previous.declaration().name().span().start),
                ));
            }
            Entry::Vacant(entry) => {
                entry.insert(functions.len());
            }
        }
    }
    Ok(LegacyFunctionBinding {
        declaration,
        name,
        visibility,
        inclusion,
    })
}

fn item_offset(item: &LegacyModuleItem<'_, '_>) -> u64 {
    match item {
        LegacyModuleItem::Function(function) => function.name().span().start,
        LegacyModuleItem::UsingAnimTree(directive) => directive.span().start,
        LegacyModuleItem::DeveloperBegin { offset } | LegacyModuleItem::DeveloperEnd { offset } => {
            *offset
        }
    }
}

fn failure(kind: SymbolErrorKind, offset: u64, previous_offset: Option<u64>) -> SymbolError {
    SymbolError {
        kind,
        offset,
        previous_offset,
    }
}
