// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::KernelErrorKind;
use crate::execution::calls::kernel_module_admission::KernelModuleAdmission;
use crate::execution::calls::{
    KernelFunctionDefinition, KernelModuleError, KernelModuleVerificationBudget,
    VerifiedKernelModule,
};
use crate::execution::kernel_error;
use crate::execution::kernel_verifier;

pub(super) fn verify(
    definitions: &[KernelFunctionDefinition<'_>],
    budget: KernelModuleVerificationBudget,
) -> Result<VerifiedKernelModule, KernelModuleError> {
    let mut admission = KernelModuleAdmission::new(definitions, budget)
        .map_err(|cause| KernelModuleError::verification(None, cause))?;
    let mut functions = Vec::new();
    functions
        .try_reserve_exact(definitions.len())
        .map_err(|_| {
            KernelModuleError::verification(
                None,
                kernel_error::general(KernelErrorKind::AllocationFailed),
            )
        })?;
    for (index, definition) in definitions.iter().enumerate() {
        let function = Some(index as u64);
        let budget = admission
            .function_budget(definition.instructions.len())
            .map_err(|cause| KernelModuleError::verification(function, cause))?;
        let proof = kernel_verifier::verify_linked(
            definition.instructions,
            definition.layout,
            budget,
            Some(definitions),
        )
        .map_err(|cause| {
            KernelModuleError::verification(
                function,
                admission.error(cause, definition.instructions.len()),
            )
        })?;
        admission
            .record(proof.usage)
            .map_err(|cause| KernelModuleError::verification(function, cause))?;
        functions.push(proof);
    }
    Ok(VerifiedKernelModule {
        functions,
        usage: admission.usage,
    })
}
