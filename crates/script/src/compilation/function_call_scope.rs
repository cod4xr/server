// SPDX-License-Identifier: AGPL-3.0-or-later
use std::collections::HashMap;

use crate::binding::{LegacyFunctionVisibility, LegacyIdentifier};

#[derive(Clone, Copy)]
pub(super) struct FunctionCallTarget {
    pub(super) function: usize,
    pub(super) visibility: LegacyFunctionVisibility,
}

pub(super) struct FunctionCallScope<'source> {
    pub(super) names: HashMap<LegacyIdentifier<'source>, FunctionCallTarget>,
    pub(super) largest_name_bytes: u64,
}

impl FunctionCallScope<'_> {
    pub(super) fn find(&self, name: LegacyIdentifier<'_>) -> Option<FunctionCallTarget> {
        if name.as_bytes().len() as u64 > self.largest_name_bytes {
            return None;
        }
        self.names.get(&name).copied()
    }
}
