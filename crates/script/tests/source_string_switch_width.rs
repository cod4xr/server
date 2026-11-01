// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Wide source case tables must keep full indices and byte identities with only one runtime string slot."
)]
use crate::{array_read_support::number, module_compile_support};
use cod4x_script::binding::LegacyDeveloperMode::Disabled;
use cod4x_script::compilation::ModuleCompilationBudget;
use cod4x_script::execution::{KernelCallBudget, KernelHeapContext, KernelValue};
use cod4x_script::heap::{HeapWorkBudget, StringHeap, StringHeapBudget};
use cod4x_script::syntax::LegacySyntaxBudget;
use std::fmt::Write;

#[test]
fn sixty_six_thousand_source_string_cases_keep_wide_rows_and_logarithmic_dispatch_without_caches() {
    let mut source = String::from("main(k){switch(k){");
    for key in (0..66_000).rev() {
        write!(source, "case \"k{key:05}\":return {key};").expect("Source case");
    }
    source.push_str("default:return -1;}}");
    let mut budget = ModuleCompilationBudget {
        max_steps: 128_000_000,
        ..ModuleCompilationBudget::default()
    };
    budget.function.max_steps = budget.max_steps;
    let module = module_compile_support::compile_with_syntax_budget(
        source.as_bytes(),
        Disabled,
        budget,
        LegacySyntaxBudget {
            max_nodes: 4_000_000,
            max_child_edges: 5_000_000,
            max_metadata_bytes: 512 * 1024 * 1024,
            ..LegacySyntaxBudget::default()
        },
    )
    .expect("Wide source label table");
    assert_eq!(
        (module.usage().switch_labels, module.usage().string_literals),
        (66_000, 0)
    );
    source.clear();
    for key in [0, 1, 32_767, 65_535, 65_999, 66_000] {
        let mut strings = StringHeap::new(StringHeapBudget {
            max_live_strings: 1,
            max_slots: 1,
            ..StringHeapBudget::default()
        })
        .expect("Only input slot");
        let input = format!("k{key:05}");
        let id = strings
            .allocate(input.as_bytes(), HeapWorkBudget::default())
            .expect("Input")
            .id;
        let mut context = KernelHeapContext::new(
            &mut strings,
            &[],
            HeapWorkBudget {
                max_scratch_bytes: 0,
                ..HeapWorkBudget::default()
            },
        );
        let result = module
            .program()
            .execute_with_heap(
                0,
                &[KernelValue::String(id)],
                KernelCallBudget::default(),
                &mut context,
            )
            .expect("Wide lookup");
        assert_eq!(result.value, number(if key < 66_000 { key } else { -1 }));
        assert!(
            result.usage.steps <= 170,
            "Dispatch must not scan the label table"
        );
        assert_eq!(context.heap_usage().slots, 1);
        assert_eq!(context.usage().peak_scratch_bytes, 0);
    }
}

#[test]
fn seventy_thousand_byte_source_labels_preserve_complete_payload_after_source_destruction() {
    let mut source = String::from("main(k){switch(k){case \"");
    source.push_str(&"x".repeat(70_000));
    source.push_str("a\":return 42;default:return 0;}}");
    let module = module_compile_support::compile(
        source.as_bytes(),
        Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Wide bytes");
    source.clear();
    assert_eq!(
        module.usage().staged_switch_bytes,
        size_of::<Vec<u8>>() as u64 + 70_001
    );
    for suffix in [b'a', b'b'] {
        let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
        let mut bytes = vec![b'x'; 70_000];
        bytes.push(suffix);
        let id = strings
            .allocate(&bytes, HeapWorkBudget::default())
            .expect("Independent input")
            .id;
        let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default());
        assert_eq!(
            module
                .program()
                .execute_with_heap(
                    0,
                    &[KernelValue::String(id)],
                    KernelCallBudget::default(),
                    &mut context
                )
                .expect("Full bytes")
                .value,
            number(if suffix == b'a' { 42 } else { 0 })
        );
    }
}
