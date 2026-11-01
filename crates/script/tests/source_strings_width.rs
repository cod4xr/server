// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests compile large and wide source literal inventories beyond native capacities."
)]
use std::fmt::Write;

use crate::module_compile_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::ModuleCompilationBudget;
use cod4x_script::execution::{KernelCallBudget, KernelHeapContext, KernelValue};
use cod4x_script::heap::{HeapWorkBudget, StringHeap, StringHeapBudget};

#[test]
fn source_literal_payloads_exceed_native_scratch_without_native_pool_or_client_capacity_claims()
-> Result<(), String> {
    let mut text = b"main(){return \"".to_vec();
    text.extend(std::iter::repeat_n(b'x', 100_000));
    text.extend_from_slice(b"\";}");
    let module = module_compile_support::compile(
        &text,
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Large source literal");
    text.fill(0);
    let mut heap = StringHeap::new(StringHeapBudget::default()).expect("Heap");
    let mut context = KernelHeapContext::new(&mut heap, &[], HeapWorkBudget::default());
    let result = module
        .program()
        .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
        .expect("Large runtime bytes");
    let KernelValue::String(id) = result.value else {
        return Err("Expected large string".into());
    };
    assert_eq!(context.get(id).expect("Owned payload"), vec![b'x'; 100_000]);
    assert_eq!(module.usage().string_literals, 1);
    Ok(())
}

#[test]
fn distinct_source_literal_rows_cross_native_string_and_short_index_boundaries()
-> Result<(), String> {
    let mut text = String::from("main(){");
    for index in 0..70_000 {
        write!(text, "value=\"v{index}\";").expect("Source buffer");
    }
    text.push_str("return value;}");
    let module = module_compile_support::compile(
        text.as_bytes(),
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Wide source table");
    assert_eq!(module.usage().string_literals, 70_000);
    assert_eq!(module.usage().verification.string_literals, 70_000);
    let mut heap = StringHeap::new(StringHeapBudget::default()).expect("Heap");
    let mut context = KernelHeapContext::new(&mut heap, &[], HeapWorkBudget::default());
    let result = module
        .program()
        .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
        .expect("Wide source execution");
    let KernelValue::String(id) = result.value else {
        return Err("Expected wide source result".into());
    };
    assert_eq!(context.get(id).expect("Final checked row"), b"v69999");
    assert_eq!(context.heap_usage().live_strings, 70_000);
    Ok(())
}
