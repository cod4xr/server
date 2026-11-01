// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests compile real owned source strings with explicit heap lifetimes."
)]
use crate::module_compile_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::ModuleCompilationBudget;
use cod4x_script::execution::{KernelCallBudget, KernelHeapContext, KernelValue};
use cod4x_script::heap::{HeapWorkBudget, StringHeap, StringHeapBudget};
use cod4x_script::legacy::LegacyNumber;

#[test]
fn source_literals_own_escaped_binary_bytes_and_survive_indirect_calls_after_source_drop()
-> Result<(), String> {
    let mut text =
        br#"main(){target=::join;return [[target]]("left\n", "right\"\\");}join(a,b){return a+b;}"#
            .to_vec();
    let module = module_compile_support::compile(
        &text,
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Quoted source and indirect call");
    text.fill(0);
    let mut heap = StringHeap::new(StringHeapBudget::default()).expect("Heap");
    let mut context = KernelHeapContext::new(&mut heap, &[], HeapWorkBudget::default());
    let result = module
        .program()
        .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
        .expect("Source-owned payload");
    let KernelValue::String(id) = result.value else {
        return Err("Expected source string".into());
    };
    assert_eq!(
        context.get(id).expect("Escaped exact bytes"),
        b"left\nright\"\\"
    );
    assert_eq!(module.usage().string_literals, 2);
    assert_eq!(module.usage().verification.string_literals, 2);
    assert_eq!(
        module.usage().staged_string_bytes,
        module.usage().verification.retained_string_bytes
    );
    let mut binary = b"main(){return \"".to_vec();
    binary.extend_from_slice(&[0xff, 0x80]);
    binary.extend_from_slice(b"\";}");
    let module = module_compile_support::compile(
        &binary,
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Binary literal source");
    binary.fill(0);
    let result = module
        .program()
        .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
        .expect("No UTF-8 repair");
    let KernelValue::String(id) = result.value else {
        return Err("Expected binary string".into());
    };
    assert_eq!(context.get(id).expect("Raw bytes"), [0xff, 0x80]);
    Ok(())
}

#[test]
fn source_literal_caches_and_local_calls_reclaim_overwritten_loop_results() -> Result<(), String> {
    let text = br#"main(){value="";for(i=0;i<1000;i++){value=combine("a","b");}return value;}combine(a,b){return a+b;}"#;
    let module = module_compile_support::compile(
        text,
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("String-producing source loop");
    let mut heap = StringHeap::new(StringHeapBudget {
        max_live_strings: 5,
        max_slots: 5,
        ..StringHeapBudget::default()
    })
    .expect("Admitted heap");
    let mut context = KernelHeapContext::new(&mut heap, &[], HeapWorkBudget::default());
    let result = module
        .program()
        .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
        .expect("Calls/locals/cache roots under pressure");
    let KernelValue::String(id) = result.value else {
        return Err("Expected final string".into());
    };
    assert_eq!(context.get(id).expect("Final bytes"), b"ab");
    assert_eq!(module.usage().string_literals, 3);
    assert_eq!(context.usage().collections, 998);
    assert_eq!(context.heap_usage().slots, 5);
    assert_eq!(context.heap_usage().payload_bytes, 6);
    Ok(())
}

#[test]
fn distinct_source_rows_compare_by_contents_and_string_tags_preserve_pure_predicates() {
    let module = module_compile_support::compile(br#"main(){if("same"!="same"){return 0;}if("x"==""){return 0;}return isint("x")+isfloat("x")+isvector("x");}"#, LegacyDeveloperMode::Disabled, ModuleCompilationBudget::default()).expect("Source string operators/predicates");
    let mut heap = StringHeap::new(StringHeapBudget::default()).expect("Heap");
    let mut context = KernelHeapContext::new(&mut heap, &[], HeapWorkBudget::default());
    assert_eq!(
        module
            .program()
            .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
            .expect("Plain equality and tags")
            .value,
        KernelValue::Number(LegacyNumber::Integer(0))
    );
}
