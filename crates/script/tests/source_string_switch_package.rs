// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Included functions retain independent case rows and literal-cache offsets under shared package admission."
)]
use crate::{array_read_support::number, package_compile_support::compile};
use cod4x_script::binding::LegacyDeveloperMode::Disabled;
use cod4x_script::compilation::ScriptPackageBudget;
use cod4x_script::execution::{KernelCallBudget, KernelHeapContext};
use cod4x_script::heap::{HeapWorkBudget, StringHeap, StringHeapBudget};

#[test]
fn package_includes_calls_and_indirect_targets_keep_function_local_label_zero_independent() {
    let inputs: &[(&[u8], &[u8])] = &[
        (b"root", br#"#include lib;main(){f=::other;return choose("alpha")+[[f]]("beta");}other(k){switch(k){case "beta":return 20;default:return -100;}}"#),
        (b"lib", br#"choose(k){switch(k){case "alpha":return 22;default:return -100;}}"#),
    ];
    let package =
        compile(inputs, Disabled, ScriptPackageBudget::default()).expect("Included switch package");
    let usage = package.usage().compilation;
    assert_eq!((usage.switch_labels, usage.string_literals), (2, 2));
    let mut strings = StringHeap::new(StringHeapBudget {
        max_live_strings: 2,
        max_slots: 2,
        ..StringHeapBudget::default()
    })
    .expect("Runtime literals only");
    let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default());
    assert_eq!(
        package
            .program()
            .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
            .expect("Per-function labels")
            .value,
        number(42)
    );
    assert_eq!(context.heap_usage().slots, 2);
    let mut exact = ScriptPackageBudget::default();
    exact.compilation.max_steps = usage.compiler_steps;
    exact.compilation.max_metadata_bytes = usage.peak_metadata_bytes;
    assert_eq!(
        compile(inputs, Disabled, exact)
            .expect("Exact package prefix and label overlap")
            .usage(),
        package.usage()
    );
    for resource in 0..2 {
        let mut short = exact;
        if resource == 0 {
            short.compilation.max_steps -= 1;
        } else {
            short.compilation.max_metadata_bytes -= 1;
        }
        compile(inputs, Disabled, short).expect_err("One less global package scope");
    }
}
