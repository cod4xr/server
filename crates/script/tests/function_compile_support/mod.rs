// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Helpers accept structurally valid synthetic single-function modules."
)]

use cod4x_script::binding::{LegacyDeveloperMode, LegacyModuleSymbols, SymbolBudget};
use cod4x_script::compilation;
use cod4x_script::compilation::{
    CompiledFunction, FunctionCompilationBudget, FunctionCompileError, FunctionSourceProfile,
};
use cod4x_script::declaration::{DeclarationBudget, LegacyModuleItem, LegacyModuleSyntax};
use cod4x_script::source::{SourceBudget, SourceText};
use cod4x_script::syntax::{LegacySyntaxBudget, LegacySyntaxMode, LegacySyntaxTree};

pub fn compile_scalar(
    bytes: &[u8],
    budget: FunctionCompilationBudget,
) -> Result<CompiledFunction, FunctionCompileError> {
    compile_profile(
        bytes,
        budget,
        FunctionSourceProfile::PureScalarReturn,
        LegacySyntaxBudget::default(),
    )
}

pub fn compile_primitive(
    bytes: &[u8],
    budget: FunctionCompilationBudget,
) -> Result<CompiledFunction, FunctionCompileError> {
    compile_primitive_with_syntax_budget(bytes, budget, LegacySyntaxBudget::default())
}

pub fn compile_primitive_with_syntax_budget(
    bytes: &[u8],
    budget: FunctionCompilationBudget,
    syntax_budget: LegacySyntaxBudget,
) -> Result<CompiledFunction, FunctionCompileError> {
    compile_profile(
        bytes,
        budget,
        FunctionSourceProfile::PrimitiveStatements,
        syntax_budget,
    )
}

fn compile_profile(
    bytes: &[u8],
    budget: FunctionCompilationBudget,
    profile: FunctionSourceProfile,
    syntax_budget: LegacySyntaxBudget,
) -> Result<CompiledFunction, FunctionCompileError> {
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
        FunctionSourceProfile::PureScalarReturn => {
            compilation::compile_scalar_return_function(function, budget)
        }
        FunctionSourceProfile::PrimitiveStatements => {
            compilation::compile_primitive_function(function, budget)
        }
    }
}
