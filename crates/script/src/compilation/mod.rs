// SPDX-License-Identifier: AGPL-3.0-or-later
//! Restricted scalar source compilation with owned verified code/source maps.
//!
//! Explicit pure-return and scalar-local statement profiles have different
//! validation contracts. General statements/values, native argument adaptation,
//! module activation, host services, scheduler, and full GSC compilation remain open.

mod compiled_scalar_function;
mod scalar_body;
mod scalar_builder;
mod scalar_compilation_budget;
mod scalar_compilation_ledger;
mod scalar_compile_error;
mod scalar_conditional;
mod scalar_expression_lowering;
mod scalar_function_compiler;
mod scalar_initialization;
mod scalar_literal;
mod scalar_local_assignment;
mod scalar_local_bindings;
mod scalar_local_compiler;
mod scalar_local_names;
mod scalar_local_statements;
mod scalar_loop_transfer;
mod scalar_program;
mod scalar_relocations;
mod scalar_statement_body;
mod scalar_task;
mod scalar_task_lowering;
mod scalar_while;

pub use compiled_scalar_function::{
    CompiledScalarFunction, ScalarCompilationUsage, ScalarSourceProfile,
};
pub use scalar_compilation_budget::ScalarCompilationBudget;
pub use scalar_compile_error::{
    ScalarCompilationResource, ScalarCompileError, ScalarCompileErrorKind,
};
pub use scalar_function_compiler::compile_scalar_function;
pub use scalar_local_compiler::compile_local_scalar_function;
