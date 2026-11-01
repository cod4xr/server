// SPDX-License-Identifier: AGPL-3.0-or-later
//! Restricted function source compilation with owned verified code/source maps.
//!
//! Explicit pure-return and primitive statement profiles have different
//! validation contracts. General statements/values, native argument adaptation,
//! module activation, host services, scheduler, and full GSC compilation remain open.

mod bound_module;
mod compiled_function;
mod function_body;
mod function_builder;
mod function_call;
mod function_call_arguments;
mod function_call_scope;
mod function_call_target;
mod function_compilation_budget;
mod function_compilation_ledger;
mod function_compile_error;
mod function_conditional;
mod function_expression_lowering;
mod function_field_assignment;
mod function_field_names;
mod function_field_update;
mod function_for;
mod function_for_clause;
mod function_for_update;
mod function_group;
mod function_initialization;
mod function_literal;
mod function_local_assignment;
mod function_local_bindings;
mod function_local_names;
mod function_local_statements;
mod function_loop_transfer;
mod function_lowering_budget;
mod function_lowering_mode;
mod function_program;
mod function_reference;
mod function_reference_value;
mod function_relocations;
mod function_statement_body;
mod function_string_literal;
mod function_switch;
mod function_task;
mod function_task_lowering;
mod function_while;
mod module_call_scope;
mod primitive_function_compiler;
mod scalar_return_compiler;

pub use bound_module::{
    CompiledModule, CompiledModuleDeclaration, CompiledModuleFunction, CompiledScriptModule,
    CompiledScriptPackage, ModuleCompilationBudget, ModuleCompilationResource,
    ModuleCompilationUsage, ModuleCompileError, ScriptModuleDefinition, ScriptPackageBudget,
    ScriptPackageError, ScriptPackageLocation, ScriptPackageUsage, compile_bound_primitive_module,
    compile_bound_primitive_package,
};
pub use compiled_function::{CompiledFunction, FunctionCompilationUsage, FunctionSourceProfile};
pub use function_compilation_budget::FunctionCompilationBudget;
pub use function_compile_error::{
    FunctionCompilationResource, FunctionCompileError, FunctionCompileErrorKind,
};
pub use function_lowering_budget::FunctionLoweringBudget;
pub use primitive_function_compiler::compile_primitive_function;
pub use scalar_return_compiler::compile_scalar_return_function;
