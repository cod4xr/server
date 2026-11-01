// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests execute wide field tables and preserve the native long-identifier case boundary."
)]
use std::fmt::Write;

use crate::module_compile_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::ModuleCompilationBudget;
use cod4x_script::execution::{KernelCallBudget, KernelHeapContext, KernelValue};
use cod4x_script::heap::{
    HeapWorkBudget, ObjectHeap, ObjectHeapBudget, StringHeap, StringHeapBudget,
};
use cod4x_script::legacy::LegacyNumber;
use cod4x_script::syntax::LegacySyntaxBudget;

#[test]
fn seventy_thousand_distinct_source_fields_execute_without_short_name_indices_or_string_caches() {
    let mut source = String::from("main(){o=spawnstruct();");
    for index in 0..70_000 {
        write!(source, "o.f{index}={index};").expect("Source buffer");
    }
    source.push_str("return o.f69999;}");
    let module = module_compile_support::compile_with_syntax_budget(
        source.as_bytes(),
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
        LegacySyntaxBudget {
            max_nodes: 2_000_000,
            ..LegacySyntaxBudget::default()
        },
    )
    .expect("Wide fields compile");
    assert_eq!(
        (
            module.usage().field_names,
            module.usage().verification.field_names,
            module.usage().string_literals
        ),
        (70_001, 70_001, 0)
    );
    source.clear();
    let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let mut objects = ObjectHeap::new(&strings, ObjectHeapBudget::default()).expect("Objects");
    let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
        .with_objects(&mut objects, &[]);
    assert_eq!(
        module
            .program()
            .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
            .expect("Wide field operands")
            .value,
        KernelValue::Number(LegacyNumber::Integer(69_999))
    );
    assert_eq!(
        context.object_heap_usage().expect("Wide store").live_fields,
        70_000
    );
    assert_eq!(context.heap_usage().live_strings, 0);
    assert_eq!(context.usage().object_collections, 0);
}

#[test]
fn field_names_fold_below_8192_bytes_and_preserve_exact_case_at_and_above_it() {
    for (length, expected) in [(8191, 1), (8192, 0), (100_000, 0)] {
        let upper = "A".repeat(length);
        let lower = "a".repeat(length);
        let source = format!("main(){{o=spawnstruct();o.{upper}=7;return isdefined(o.{lower});}}");
        let module = module_compile_support::compile(
            source.as_bytes(),
            LegacyDeveloperMode::Disabled,
            ModuleCompilationBudget::default(),
        )
        .expect("Untruncated field name");
        assert_eq!(
            module.usage().staged_field_bytes,
            2 * (size_of::<Vec<u8>>() + length) as u64
        );
        let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
        let mut objects = ObjectHeap::new(&strings, ObjectHeapBudget::default()).expect("Objects");
        let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
            .with_objects(&mut objects, &[]);
        assert_eq!(
            module
                .program()
                .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
                .expect("Native canonicalization boundary")
                .value,
            KernelValue::Number(LegacyNumber::Integer(expected))
        );
        assert_eq!(
            context
                .object_heap_usage()
                .expect("Owned complete key")
                .key_bytes,
            length as u64
        );
    }
}
