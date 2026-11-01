// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests exercise rightmost string argument errors and expanded argument counts."
)]
use crate::module_compile_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::ModuleCompilationBudget;
use cod4x_script::execution::{KernelCallBudget, KernelErrorKind, KernelHeapContext};
use cod4x_script::heap::{HeapWorkBudget, StringHeap, StringHeapBudget};

#[test]
fn rightmost_string_argument_error_precedes_builtin_arity_and_left_argument_evaluation()
-> Result<(), String> {
    let source = br#"main(){return isstring(left(),right());}left(){return "PrivateLeft"+1;}right(){return "PrivateRight"/1;}"#;
    let module = module_compile_support::compile(
        source,
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Typed source arguments");
    let mut heap = StringHeap::new(StringHeapBudget::default()).expect("Heap");
    let mut context = KernelHeapContext::new(&mut heap, &[], HeapWorkBudget::default());
    let error = module
        .program()
        .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
        .expect_err("Rightmost argument failure");
    assert_eq!(error.function, Some(2));
    assert_eq!(error.cause.kind, KernelErrorKind::UnmatchingTypes);
    assert_eq!(context.heap_usage().live_strings, 1);
    let span = module
        .instruction_span(2, error.cause.instruction.ok_or("Instruction")?)
        .ok_or("Map")?;
    assert_eq!(
        &source[usize::try_from(span.start).expect("Offset")
            ..usize::try_from(span.end).expect("Offset")],
        b"/"
    );
    assert!(!format!("{error:?} {context:?}").contains("Private"));
    Ok(())
}

#[test]
fn source_predicate_evaluates_all_256_string_arguments_without_byte_count_truncation() {
    let source = format!(
        "main(){{return isstring({});}}",
        vec!["\"x\""; 256].join(",")
    );
    let module = module_compile_support::compile(
        source.as_bytes(),
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Wide builtin call");
    let mut heap = StringHeap::new(StringHeapBudget::default()).expect("Heap");
    let mut context = KernelHeapContext::new(&mut heap, &[], HeapWorkBudget::default());
    let error = module
        .program()
        .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
        .expect_err("Runtime exact-one arity");
    assert_eq!(
        error.cause.kind,
        KernelErrorKind::ArgumentCount {
            expected: 1,
            actual: 256
        }
    );
    assert_eq!(context.heap_usage().live_strings, 256);
    assert_eq!(context.heap_usage().payload_bytes, 256);
    assert_eq!(module.usage().string_literals, 256);
    assert_eq!(error.peak_frames, 1);
}
