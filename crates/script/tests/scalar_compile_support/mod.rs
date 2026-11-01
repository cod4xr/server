// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Helpers accept structurally valid synthetic single-function modules."
)]

use cod4x_script::binding::{LegacyDeveloperMode, LegacyModuleSymbols, SymbolBudget};
use cod4x_script::compilation;
use cod4x_script::compilation::{
    CompiledScalarFunction, ScalarCompilationBudget, ScalarCompileError, ScalarSourceProfile,
};
use cod4x_script::declaration::{DeclarationBudget, LegacyModuleItem, LegacyModuleSyntax};
use cod4x_script::source::{SourceBudget, SourceText};
use cod4x_script::syntax::{LegacySyntaxBudget, LegacySyntaxMode, LegacySyntaxTree};

pub fn compile(
    bytes: &[u8],
    budget: ScalarCompilationBudget,
) -> Result<CompiledScalarFunction, ScalarCompileError> {
    compile_profile(
        bytes,
        budget,
        ScalarSourceProfile::PureReturn,
        LegacySyntaxBudget::default(),
    )
}

pub fn compile_local(
    bytes: &[u8],
    budget: ScalarCompilationBudget,
) -> Result<CompiledScalarFunction, ScalarCompileError> {
    compile_local_with_syntax_budget(bytes, budget, LegacySyntaxBudget::default())
}

pub fn compile_local_with_syntax_budget(
    bytes: &[u8],
    budget: ScalarCompilationBudget,
    syntax_budget: LegacySyntaxBudget,
) -> Result<CompiledScalarFunction, ScalarCompileError> {
    compile_profile(
        bytes,
        budget,
        ScalarSourceProfile::LocalStatements,
        syntax_budget,
    )
}

fn compile_profile(
    bytes: &[u8],
    budget: ScalarCompilationBudget,
    profile: ScalarSourceProfile,
    syntax_budget: LegacySyntaxBudget,
) -> Result<CompiledScalarFunction, ScalarCompileError> {
    let source = SourceText::index(bytes, SourceBudget::default()).expect("Source");
    let tree = LegacySyntaxTree::parse_structure(&source, LegacySyntaxMode::Script, syntax_budget)
        .expect("Tree");
    let module =
        LegacyModuleSyntax::lower(&tree, DeclarationBudget::default()).expect("Declarations");
    let _symbols = LegacyModuleSymbols::bind(
        &module,
        LegacyDeveloperMode::Disabled,
        SymbolBudget::default(),
    )
    .expect("Local binding");
    let function = module
        .items()
        .iter()
        .find_map(|item| match item {
            LegacyModuleItem::Function(function) => Some(function),
            _ => None,
        })
        .expect("Single synthetic function");
    match profile {
        ScalarSourceProfile::PureReturn => compilation::compile_scalar_function(function, budget),
        ScalarSourceProfile::LocalStatements => {
            compilation::compile_local_scalar_function(function, budget)
        }
    }
}
