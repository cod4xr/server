// SPDX-License-Identifier: AGPL-3.0-or-later
//! Restricted pure scalar source compilation with owned verified code/source maps.
//!
//! No parameters, general statements/values, module activation, host services,
//! scheduler, native bytecode ABI, or full GSC compiler/VM support is implied.

mod compiled_scalar_function;
mod scalar_body;
mod scalar_builder;
mod scalar_compilation_budget;
mod scalar_compilation_ledger;
mod scalar_compile_error;
mod scalar_expression_lowering;
mod scalar_function_compiler;
mod scalar_literal;
mod scalar_task;

pub use compiled_scalar_function::{CompiledScalarFunction, ScalarCompilationUsage};
pub use scalar_compilation_budget::ScalarCompilationBudget;
pub use scalar_compile_error::{
    ScalarCompilationResource, ScalarCompileError, ScalarCompileErrorKind,
};
pub use scalar_function_compiler::compile_scalar_function;
