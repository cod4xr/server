// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests compile real bounded string access modules."
)]
use crate::module_compile_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::ModuleCompilationBudget;
use cod4x_script::execution::{KernelCallBudget, KernelErrorKind, KernelHeapContext, KernelValue};
use cod4x_script::heap::{HeapWorkBudget, StringHeap, StringHeapBudget};

#[test]
fn source_calls_index_before_object_and_size_failures_keep_private_operator_maps()
-> Result<(), String> {
    for (text, entry, expected, token) in [
        (br#"main(){return object()[index()];}object(){return 1/0;}index(){return "Private"[99];}"#.as_slice(), 2,
            KernelErrorKind::StringIndexOutOfRange, b"[".as_slice()),
        (br#"main(){return "Private"[1.0];}"#, 0, KernelErrorKind::ExpectedStringIndex, b"["),
        (br#"main(){return &"Private".size;}"#, 0, KernelErrorKind::ExpectedSized, b".size"),
        (br"main(){return (1,2,3).size;}", 0, KernelErrorKind::ExpectedSized, b".size"),
    ] {
        let module = module_compile_support::compile(text, LegacyDeveloperMode::Disabled, ModuleCompilationBudget::default()).expect("Access source");
        let mut heap = StringHeap::new(StringHeapBudget::default()).expect("Heap");
        let mut context = KernelHeapContext::new(&mut heap, &[], HeapWorkBudget::default());
        let error = module.program().execute_with_heap(0, &[], KernelCallBudget::default(), &mut context).expect_err("Located access failure");
        assert_eq!(error.cause.kind, expected);
        assert_eq!(error.function, Some(entry));
        let span = module.instruction_span(entry, error.cause.instruction.ok_or("Missing PC")?).ok_or("Missing span")?;
        assert_eq!(&text[usize::try_from(span.start).expect("Offset")..usize::try_from(span.end).expect("Offset")], token);
        assert!(!format!("{error:?} {module:?} {context:?}").contains("Private"));
    }
    Ok(())
}

#[test]
fn source_size_and_byte_index_exceed_old_short_width_without_utf8_conversion() -> Result<(), String>
{
    let mut text = b"main(){value=\"".to_vec();
    text.extend(vec![b'x'; 70_000]);
    text.extend_from_slice(b"\";if(value.size!=70000){return undefined;}return value[69999];}");
    let module = module_compile_support::compile(
        &text,
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Wide access source");
    text.fill(0);
    let mut heap = StringHeap::new(StringHeapBudget::default()).expect("Heap");
    let mut context = KernelHeapContext::new(&mut heap, &[], HeapWorkBudget::default());
    let result = module
        .program()
        .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
        .expect("Wide access");
    let KernelValue::String(id) = result.value else {
        return Err("Expected indexed string".into());
    };
    assert_eq!(context.get(id).expect("Byte"), b"x");
    let module = module_compile_support::compile(
        "main(){return \"é\"[1];}".as_bytes(),
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("UTF-8 source bytes");
    let result = module
        .program()
        .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
        .expect("Index a byte, not a character");
    let KernelValue::String(id) = result.value else {
        return Err("Missing raw byte".into());
    };
    assert_eq!(context.get(id).expect("Raw byte"), [0xa9]);
    Ok(())
}

#[test]
fn source_index_calls_reclaim_overwritten_bytes_while_preserving_caller_and_cache_roots()
-> Result<(), String> {
    let text = br#"main(){value="";for(i=0;i<1000;i++){value=pick("ab",i%2);}return value;}pick(text,i){return text[i];}"#;
    let module = module_compile_support::compile(
        text,
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Indexed source loop");
    let mut heap = StringHeap::new(StringHeapBudget {
        max_live_strings: 4,
        max_slots: 4,
        ..StringHeapBudget::default()
    })
    .expect("Heap");
    let mut context = KernelHeapContext::new(&mut heap, &[], HeapWorkBudget::default());
    let result = module
        .program()
        .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
        .expect("Rooted index loop");
    let KernelValue::String(id) = result.value else {
        return Err("Missing final byte".into());
    };
    assert_eq!(context.get(id).expect("Final byte"), b"b");
    assert_eq!(context.heap_usage().slots, 4);
    assert_eq!(context.heap_usage().payload_bytes, 4);
    assert_eq!(context.usage().collections, 998);
    Ok(())
}
