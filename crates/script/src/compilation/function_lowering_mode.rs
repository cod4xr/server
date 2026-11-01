// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::binding::{LegacyDeveloperMode, LegacyFunctionVisibility};
use crate::compilation::FunctionSourceProfile;
use crate::execution::{KernelExecutionBudget, KernelVerificationBudget};

#[derive(Clone, Copy)]
pub(super) struct StandaloneFunctionMode {
    pub(super) profile: FunctionSourceProfile,
    pub(super) verification: KernelVerificationBudget,
    pub(super) validation: KernelExecutionBudget,
}
use crate::compilation::function_call_scope::FunctionCallScope;

#[derive(Clone, Copy)]
pub(super) enum FunctionLoweringMode<'scope, 'source> {
    Standalone(StandaloneFunctionMode),
    BoundModule {
        scope: &'scope FunctionCallScope<'source>,
        caller: LegacyFunctionVisibility,
        module: usize,
        developer: LegacyDeveloperMode,
    },
}

impl FunctionLoweringMode<'_, '_> {
    pub(super) fn primitive(self) -> bool {
        match self {
            Self::Standalone(mode) => mode.profile == FunctionSourceProfile::PrimitiveStatements,
            Self::BoundModule { .. } => true,
        }
    }

    pub(super) fn bound(self) -> bool {
        matches!(self, Self::BoundModule { .. })
    }

    pub(super) fn developer(self) -> LegacyDeveloperMode {
        match self {
            Self::BoundModule { developer, .. } => developer,
            Self::Standalone(_) => LegacyDeveloperMode::Disabled,
        }
    }

    pub(super) fn developer_function(self) -> bool {
        matches!(
            self,
            Self::BoundModule {
                caller: LegacyFunctionVisibility::DeveloperOnly,
                ..
            }
        )
    }

    pub(super) fn standalone(self) -> Option<StandaloneFunctionMode> {
        match self {
            Self::Standalone(profile) => Some(profile),
            Self::BoundModule { .. } => None,
        }
    }
}
