// SPDX-License-Identifier: AGPL-3.0-or-later
use std::ops::Range;

use crate::binding::{LegacyFunctionInclusion, LegacyModuleSymbols};
use crate::compilation::bound_module::module_compilation_ledger::ModuleCompilationLedger;
use crate::compilation::bound_module::module_inventory;
use crate::compilation::bound_module::module_program;
use crate::compilation::bound_module::module_storage;
use crate::compilation::bound_module::staged_function;
use crate::compilation::bound_module::{
    CompiledModule, ModuleCompilationBudget, ModuleCompileError,
};
use crate::compilation::function_lowering_mode::FunctionLoweringMode;

/// Compiles active local definitions through a complete primitive call inventory.
///
/// Reuses primitive statements with immediate local/indirect calls, local function
/// references and sealed builtins. Arguments lower rightmost-first before
/// indirect target expressions; invocation checks arity.
/// Bound source includes metered local lifetimes and nonnested indexed assignment
/// through initialized plain locals. Heap operations require an explicit runtime
/// heap context; stored Array values and root/game writes remain unsupported.
/// Ordinary code cannot reference developer-only targets. Disabled declarations
/// remain metadata, including unsupported bodies that are not compiled. No source/
/// tree/name/symbol borrow escapes. Empty/all-disabled inventories contain no entry.
/// This API does not process imports/animation directives, methods/thread calls,
/// general builtins, host services, activation or full GSC. Existing isolated source
/// profiles and restricted statement/for-clause policies remain unchanged.
///
/// # Errors
/// Rejects unknown/unsupported/developer calls, individual source/binding/flow
/// failures, aggregate/per-function budget exhaustion, kernel proof and allocation/
/// size failures. No partial module or external mutation escapes. Names/values
/// never enter errors; compiler and kernel proof work remain separate scopes.
///
/// ```
/// use cod4x_script::binding::{LegacyDeveloperMode, LegacyModuleSymbols, SymbolBudget};
/// use cod4x_script::compilation::{self, ModuleCompilationBudget};
/// use cod4x_script::declaration::{DeclarationBudget, LegacyModuleSyntax};
/// use cod4x_script::execution::{KernelCallBudget, KernelValue};
/// use cod4x_script::legacy::LegacyNumber;
/// use cod4x_script::source::{SourceBudget, SourceText};
/// use cod4x_script::syntax::{LegacySyntaxBudget, LegacySyntaxMode, LegacySyntaxTree};
///
/// let source = SourceText::index(b"main(){return twice(21);}twice(n){return n*2;}",
///     SourceBudget::default())?;
/// let tree = LegacySyntaxTree::parse_structure(&source, LegacySyntaxMode::Script,
///     LegacySyntaxBudget::default())?;
/// let syntax = LegacyModuleSyntax::lower(&tree, DeclarationBudget::default())?;
/// let symbols = LegacyModuleSymbols::bind(&syntax, LegacyDeveloperMode::Disabled,
///     SymbolBudget::default())?;
/// let compiled = compilation::compile_bound_primitive_module(&symbols,
///     ModuleCompilationBudget::default())?;
/// let result = compiled.program().execute(0, &[], KernelCallBudget::default())?;
/// assert_eq!(result.value, KernelValue::Number(LegacyNumber::Integer(42)));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn compile_bound_primitive_module(
    symbols: &LegacyModuleSymbols<'_, '_, '_>,
    budget: ModuleCompilationBudget,
) -> Result<CompiledModule, ModuleCompileError> {
    let span = module_span(symbols);
    let mut ledger = ModuleCompilationLedger::new(symbols, budget, span.clone())
        .map_err(|cause| ModuleCompileError::at(None, cause))?;
    let inventory = module_inventory::build(symbols, &mut ledger, span.clone())?;
    let mut code = module_storage::reserve(ledger.usage.functions, span.clone())
        .map_err(|cause| ModuleCompileError::at(None, ledger.locate(cause)))?;
    let mut functions = module_storage::reserve(ledger.usage.functions, span.clone())
        .map_err(|cause| ModuleCompileError::at(None, ledger.locate(cause)))?;
    for (index, binding) in symbols.functions().iter().copied().enumerate() {
        ledger
            .charge(1, binding.declaration().span())
            .map_err(|cause| ModuleCompileError::at(Some(index), cause))?;
        if binding.inclusion() != LegacyFunctionInclusion::Active {
            continue;
        }
        let limits = ledger
            .function_budget(binding.declaration().span())
            .map_err(|cause| ModuleCompileError::at(Some(index), ledger.locate(cause)))?;
        let mode = FunctionLoweringMode::BoundModule {
            scope: &inventory.scope,
            caller: binding.visibility(),
            module: 0,
        };
        let (staged, map) = staged_function::lower(binding.declaration(), index, mode, limits)
            .map_err(|cause| ModuleCompileError::at(Some(index), ledger.function_error(cause)))?;
        ledger
            .record(map.usage, binding.declaration().span())
            .map_err(|cause| ModuleCompileError::at(Some(index), ledger.locate(cause)))?;
        code.push(staged);
        functions.push(map);
    }
    let program = module_program::verify(
        &code,
        &mut functions,
        &inventory.declarations,
        &mut ledger,
        span,
    )?;
    Ok(CompiledModule {
        program,
        declarations: inventory.declarations,
        functions,
        usage: ledger.usage,
    })
}

fn module_span(symbols: &LegacyModuleSymbols<'_, '_, '_>) -> Range<u64> {
    let functions = symbols.functions();
    match (functions.first(), functions.last()) {
        (Some(first), Some(last)) => {
            first.declaration().span().start..last.declaration().span().end
        }
        _ => 0..0,
    }
}
