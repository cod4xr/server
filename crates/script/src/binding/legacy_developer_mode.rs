// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::binding::{LegacyFunctionInclusion, LegacyFunctionVisibility};

/// Explicit source compiler developer-script mode, not a wire/client capability.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyDeveloperMode {
    /// Developer-only definitions remain recorded but are excluded from lookup.
    Disabled,
    /// Developer-only definitions enter lookup with their visibility retained.
    Enabled,
}

impl LegacyDeveloperMode {
    pub(super) fn inclusion(self, visibility: LegacyFunctionVisibility) -> LegacyFunctionInclusion {
        if self == Self::Disabled && visibility == LegacyFunctionVisibility::DeveloperOnly {
            LegacyFunctionInclusion::DeveloperDisabled
        } else {
            LegacyFunctionInclusion::Active
        }
    }
}
