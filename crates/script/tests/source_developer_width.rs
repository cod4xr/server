// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Developer validation and cleanup use flat wide scopes without native short counts or host recursion."
)]
use crate::{
    array_read_support::number, module_compile_support, source_developer_support::compile,
};
use cod4x_script::binding::{
    LegacyDeveloperMode,
    LegacyDeveloperMode::{Disabled, Enabled},
};
use cod4x_script::compilation::{CompiledModule, ModuleCompilationBudget};
use cod4x_script::execution::KernelCallBudget;
use cod4x_script::syntax::LegacySyntaxBudget;

fn wide(source: &[u8], mode: LegacyDeveloperMode) -> CompiledModule {
    module_compile_support::compile_with_syntax_budget(
        source,
        mode,
        ModuleCompilationBudget::default(),
        LegacySyntaxBudget {
            max_nodes: 4_000_000,
            max_child_edges: 5_000_000,
            max_metadata_bytes: 512 * 1024 * 1024,
            ..LegacySyntaxBudget::default()
        },
    )
    .expect("Explicitly admitted wide syntax inventory")
}

#[test]
fn seventy_thousand_developer_regions_retain_only_selected_code_and_no_disabled_literal_cache() {
    let mut source = String::from("main(){x=0;");
    for _ in 0..70_000 {
        source.push_str("/# x+=1; #/");
    }
    source.push_str("return x;}");
    for (mode, expected) in [(Enabled, 70_000), (Disabled, 0)] {
        let module = wide(source.as_bytes(), mode);
        let result = module
            .program()
            .execute(0, &[], KernelCallBudget::default())
            .expect("Wide regions");
        assert_eq!(result.value, number(expected));
        if mode == Disabled {
            assert!(module.usage().instructions < 10);
        }
    }
    let mut discarded = String::from("main(){");
    for _ in 0..70_000 {
        discarded.push_str("/# missing(\"Private\"); #/");
    }
    discarded.push_str("return 42;}");
    let module = wide(discarded.as_bytes(), Disabled);
    assert_eq!(
        (module.usage().string_literals, module.usage().instructions),
        (0, 2)
    );
    assert_eq!(
        module
            .program()
            .execute(0, &[], KernelCallBudget::default())
            .expect("No heap or link authority")
            .value,
        number(42)
    );
}

#[test]
fn twelve_thousand_nested_if_contexts_interleave_developer_regions_without_recursive_ownership() {
    let mut source = String::from("main(){x=0;");
    for _ in 0..12_000 {
        source.push_str("if(1){/# x+=1; #/");
    }
    for _ in 0..12_000 {
        source.push('}');
    }
    source.push_str("return x;}");
    for (mode, expected) in [(Enabled, 12_000), (Disabled, 0)] {
        let module = compile(source.as_bytes(), mode);
        assert_eq!(
            module
                .program()
                .execute(0, &[], KernelCallBudget::default())
                .expect("Flat scopes")
                .value,
            number(expected)
        );
        let mut short = ModuleCompilationBudget::default();
        short.function.max_task_frames = 32;
        module_compile_support::compile(source.as_bytes(), mode, short)
            .expect_err("Deep task failure drops flat state");
    }
}
