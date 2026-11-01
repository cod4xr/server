// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Helpers accept syntactically valid synthetic local source modules."
)]

use cod4x_script::binding::{LegacyDeveloperMode, LegacyModuleSymbols, SymbolBudget};
use cod4x_script::compilation::{
    self, CompiledModule, ModuleCompilationBudget, ModuleCompileError,
};
use cod4x_script::declaration::{DeclarationBudget, LegacyModuleSyntax};
use cod4x_script::source::{SourceBudget, SourceText};
use cod4x_script::syntax::{LegacySyntaxBudget, LegacySyntaxMode, LegacySyntaxTree};

pub fn compile(
    bytes: &[u8],
    mode: LegacyDeveloperMode,
    budget: ModuleCompilationBudget,
) -> Result<CompiledModule, ModuleCompileError> {
    compile_with_syntax_budget(bytes, mode, budget, LegacySyntaxBudget::default())
}

pub fn compile_with_syntax_budget(
    bytes: &[u8],
    mode: LegacyDeveloperMode,
    budget: ModuleCompilationBudget,
    syntax_budget: LegacySyntaxBudget,
) -> Result<CompiledModule, ModuleCompileError> {
    let source = SourceText::index(bytes, SourceBudget::default()).expect("Source");
    let tree = LegacySyntaxTree::parse_structure(&source, LegacySyntaxMode::Script, syntax_budget)
        .expect("Tree");
    let module =
        LegacyModuleSyntax::lower(&tree, DeclarationBudget::default()).expect("Declarations");
    let symbols =
        LegacyModuleSymbols::bind(&module, mode, SymbolBudget::default()).expect("Local binding");
    compilation::compile_bound_primitive_module(&symbols, budget)
}
