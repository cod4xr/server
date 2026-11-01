// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Synthetic developer fixtures compile under an explicit module policy."
)]
use crate::module_compile_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::{CompiledModule, ModuleCompilationBudget};

pub fn compile(source: &[u8], mode: LegacyDeveloperMode) -> CompiledModule {
    module_compile_support::compile(source, mode, ModuleCompilationBudget::default())
        .expect("Developer source profile")
}
