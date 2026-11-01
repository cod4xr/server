// SPDX-License-Identifier: AGPL-3.0-or-later
//! Owned verified primitive execution, not a GSC source compiler or complete VM.
//!
//! Instructions are typed internal data, with no serialized/native bytecode ABI.
//! This kernel owns no objects, strings, call frames, scheduler, host services,
//! filesystem, logging, or server state. Legacy numeric FP assumptions still apply.

mod kernel_budget;
mod kernel_comparison_values;
mod kernel_error;
mod kernel_executor;
mod kernel_flow_state;
mod kernel_function_layout;
mod kernel_index_values;
mod kernel_instruction;
mod kernel_locals;
mod kernel_numeric_values;
mod kernel_operations;
mod kernel_stack;
mod kernel_value;
mod kernel_verification_ledger;
mod kernel_verifier;
mod verified_kernel_program;

pub use kernel_budget::{KernelExecutionBudget, KernelVerificationBudget};
pub use kernel_error::{KernelError, KernelErrorKind, KernelResource};
pub use kernel_function_layout::KernelFunctionLayout;
pub use kernel_instruction::{KernelBranchCondition, KernelInstruction};
pub use kernel_value::KernelValue;
pub use verified_kernel_program::{KernelOutcome, KernelProgramUsage, VerifiedKernelProgram};
