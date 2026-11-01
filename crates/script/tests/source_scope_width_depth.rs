// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Wide/deep scopes must retain complete indices and iterative cleanup."
)]
use crate::{module_compile_support, source_scope_support};
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::ModuleCompilationBudget;
use cod4x_script::execution::KernelCallBudget;
use cod4x_script::heap::HeapWorkBudget;
use cod4x_script::syntax::LegacySyntaxBudget;
use std::fmt::Write;

#[test]
fn seventy_thousand_private_aliases_are_removed_without_a_byte_or_short_count() {
    let mut source = String::from("main(){a=[];if(1){");
    for index in 0..70_000 {
        write!(source, "b{index}=a;").expect("Wide source");
    }
    source.push_str("}c=[];return 0;}");
    let module = module_compile_support::compile_with_syntax_budget(
        source.as_bytes(),
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
        LegacySyntaxBudget {
            max_nodes: 4_000_000,
            max_child_edges: 5_000_000,
            max_metadata_bytes: 512 * 1024 * 1024,
            ..LegacySyntaxBudget::default()
        },
    )
    .expect("Wide scoped aliases");
    let run = source_scope_support::run(
        &module,
        70_001,
        KernelCallBudget::default(),
        HeapWorkBudget::default(),
    );
    run.result.expect("All private aliases removed before c");
    assert_eq!(run.objects.array_value_slots, 70_001);
    assert_eq!(run.objects.live_arrays, 2);
}

#[test]
fn twelve_thousand_nested_bound_loops_release_planning_contexts_iteratively() {
    let mut source = String::from("main(){a=[];");
    for index in 0..12_000 {
        source.push_str(if index % 2 == 0 {
            "while(0){"
        } else {
            "for(;0;){"
        });
    }
    source.push_str("b=a;");
    for _ in 0..12_000 {
        source.push('}');
    }
    source.push_str("c=[];return 0;}");
    let run = source_scope_support::succeeds(source.as_bytes(), 2);
    assert_eq!(run.objects.array_value_slots, 2);
}

#[test]
fn twelve_thousand_nested_bound_branches_plan_lower_and_clean_iteratively() {
    let mut source = String::from("main(){a=[];");
    for _ in 0..12_000 {
        source.push_str("if(1){");
    }
    source.push_str("b=a;");
    for _ in 0..12_000 {
        source.push('}');
    }
    source.push_str("c=[];return 0;}");
    let run = source_scope_support::succeeds(source.as_bytes(), 2);
    assert_eq!(run.objects.array_value_slots, 2);
}
