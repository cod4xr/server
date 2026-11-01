// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Helpers construct admitted source/tree/symbol inputs and then return only owned packages."
)]
use cod4x_script::binding::{LegacyDeveloperMode, LegacyModuleSymbols, SymbolBudget};
use cod4x_script::compilation::{
    self, CompiledScriptPackage, ScriptModuleDefinition, ScriptPackageBudget, ScriptPackageError,
};
use cod4x_script::declaration::{DeclarationBudget, LegacyModuleSyntax};
use cod4x_script::source::{SourceBudget, SourceText};
use cod4x_script::syntax::{LegacySyntaxBudget, LegacySyntaxMode, LegacySyntaxTree};

pub fn compile(
    inputs: &[(&[u8], &[u8])],
    mode: LegacyDeveloperMode,
    budget: ScriptPackageBudget,
) -> Result<CompiledScriptPackage, ScriptPackageError> {
    let sources: Vec<_> = inputs
        .iter()
        .map(|(_, bytes)| SourceText::index(bytes, SourceBudget::default()).expect("Source"))
        .collect();
    let trees: Vec<_> = sources
        .iter()
        .map(|source| {
            LegacySyntaxTree::parse_structure(
                source,
                LegacySyntaxMode::Script,
                LegacySyntaxBudget::default(),
            )
            .expect("Tree")
        })
        .collect();
    let modules: Vec<_> = trees
        .iter()
        .map(|tree| {
            LegacyModuleSyntax::lower(tree, DeclarationBudget::default()).expect("Declarations")
        })
        .collect();
    let symbols: Vec<_> = modules
        .iter()
        .map(|module| {
            LegacyModuleSymbols::bind(module, mode, SymbolBudget::default()).expect("Local symbols")
        })
        .collect();
    let definitions: Vec<_> = inputs
        .iter()
        .zip(&symbols)
        .zip(&modules)
        .map(|(((name, _), symbols), module)| {
            ScriptModuleDefinition::new(name, symbols).with_includes(module.includes())
        })
        .collect();
    compilation::compile_bound_primitive_package(&definitions, budget)
}
