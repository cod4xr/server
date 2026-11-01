// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::execution::KernelErrorKind;
use crate::execution::calls::kernel_module_admission::KernelModuleAdmission;
use crate::execution::calls::{
    KernelFunctionDefinition, KernelModuleError, KernelModuleVerificationBudget,
    VerifiedKernelModule,
};
use crate::execution::kernel_error;
use crate::execution::kernel_module_identity::KernelModuleIdentity;
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
    let mut literal_offset = 0_usize;
    for (index, definition) in definitions.iter().enumerate() {
        let function = Some(index as u64);
        let budget = admission
            .function_budget(*definition)
            .map_err(|cause| KernelModuleError::verification(function, cause))?;
        let mut proof = kernel_verifier::verify_linked(
            definition.instructions,
            definition.layout,
            definition.string_literals,
            budget,
            Some(definitions),
        )
        .map_err(|cause| {
            KernelModuleError::verification(function, admission.error(cause, budget))
        })?;
        proof.literal_offset = literal_offset;
        literal_offset = literal_offset
            .checked_add(proof.string_literals.len())
            .ok_or_else(|| {
                KernelModuleError::verification(
                    function,
                    kernel_error::general(KernelErrorKind::SizeOverflow)
                        .with_usage(admission.usage.verification_steps, 0),
                )
            })?;
        admission
            .record(proof.usage)
            .map_err(|cause| KernelModuleError::verification(function, cause))?;
        functions.push(proof);
    }
    Ok(VerifiedKernelModule {
        identity: KernelModuleIdentity::allocate().map_err(|kind| {
            KernelModuleError::verification(
                None,
                kernel_error::general(kind).with_usage(admission.usage.verification_steps, 0),
            )
        })?,
        functions,
        usage: admission.usage,
    })
}
