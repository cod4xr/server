// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests execute localized source alongside real plain-string allocation pressure."
)]
use std::fmt::Write;

use crate::module_compile_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::ModuleCompilationBudget;
use cod4x_script::execution::{KernelCallBudget, KernelHeapContext, KernelValue};
use cod4x_script::heap::{HeapWorkBudget, StringHeap, StringHeapBudget};

#[test]
fn localized_loop_values_and_cached_keys_survive_repeated_plain_allocation_collection()
-> Result<(), String> {
    let source = br#"main(){value=&"";for(i=0;i<1000;i++){value=copy(&"MENU");waste="a"+"b";}return value;}copy(x){return x;}"#;
    let module = module_compile_support::compile(
        source,
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Localized source loop");
    let mut heap = StringHeap::new(StringHeapBudget {
        max_live_strings: 6,
        max_slots: 6,
        ..StringHeapBudget::default()
    })
    .expect("Small heap");
    let mut context = KernelHeapContext::new(&mut heap, &[], HeapWorkBudget::default());
    let result = module
        .program()
        .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
        .expect("Localized roots under pressure");
    let KernelValue::LocalizedString(id) = result.value else {
        return Err("Expected localized key".into());
    };
    assert_eq!(context.get(id).expect("No translation/copy loss"), b"MENU");
    assert_eq!(context.usage().collections, 998);
    assert_eq!(context.heap_usage().live_strings, 6);
    assert_eq!(context.heap_usage().payload_bytes, 10);
    Ok(())
}

#[test]
fn mixed_plain_localized_source_tables_exceed_native_pool_and_short_indices() -> Result<(), String>
{
    let mut source = String::from("main(){");
    for index in 0..70_000 {
        let prefix = if index % 2 == 0 { "" } else { "&" };
        write!(source, "value={prefix}\"k{index}\";").expect("Source buffer");
    }
    source.push_str("return value;}");
    let module = module_compile_support::compile(
        source.as_bytes(),
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Mixed wide source");
    let mut heap = StringHeap::new(StringHeapBudget::default()).expect("Heap");
    let mut context = KernelHeapContext::new(&mut heap, &[], HeapWorkBudget::default());
    let result = module
        .program()
        .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
        .expect("Wide tagged table");
    let KernelValue::LocalizedString(id) = result.value else {
        return Err("Expected final localized row".into());
    };
    assert_eq!(context.get(id).expect("No truncated index"), b"k69999");
    assert_eq!(module.usage().string_literals, 70_000);
    assert_eq!(context.heap_usage().live_strings, 70_000);
    Ok(())
}
