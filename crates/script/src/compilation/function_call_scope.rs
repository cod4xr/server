// SPDX-License-Identifier: AGPL-3.0-or-later
use std::collections::HashMap;

use crate::binding::{LegacyFunctionVisibility, LegacyIdentifier};
use crate::compilation::module_call_scope::ModuleCallScope;

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub(super) struct FunctionCallName<'source> {
    pub(super) module: usize,
    pub(super) name: LegacyIdentifier<'source>,
}

#[derive(Clone, Copy)]
pub(super) struct FunctionCallTarget {
    pub(super) function: usize,
    pub(super) visibility: LegacyFunctionVisibility,
}

pub(super) struct FunctionCallScope<'source> {
    pub(super) names: HashMap<FunctionCallName<'source>, FunctionCallTarget>,
    pub(super) modules: Option<ModuleCallScope<'source>>,
    pub(super) largest_name_bytes: u64,
}

impl FunctionCallScope<'_> {
    pub(super) fn find(
        &self,
        module: usize,
        name: LegacyIdentifier<'_>,
    ) -> Option<FunctionCallTarget> {
        if name.as_bytes().len() as u64 > self.largest_name_bytes {
            return None;
        }
        self.names.get(&FunctionCallName { module, name }).copied()
    }
}
