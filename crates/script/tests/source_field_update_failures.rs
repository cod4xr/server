// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests inspect failure timing, unchanged fields and private source maps."
)]
use crate::module_compile_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::ModuleCompilationBudget;
use cod4x_script::execution::{KernelCallBudget, KernelErrorKind, KernelHeapContext, KernelValue};
use cod4x_script::heap::{
    HeapWorkBudget, ObjectHeap, ObjectHeapBudget, StringHeap, StringHeapBudget,
};
use cod4x_script::legacy::{LegacyNumber, LegacyNumberError};

#[test]
fn failed_compound_operator_does_not_evaluate_the_second_receiver_or_overwrite_the_field()
-> Result<(), String> {
    let source = b"main(s){receiver(s).PrivateField+=rhs(s);}receiver(s){s.trace=s.trace*10+1;return s;}rhs(s){s.trace=s.trace*10+2;return 1;}";
    let module = module_compile_support::compile(
        source,
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Located compound failure");
    let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let held = strings
        .allocate(b"PrivatePayload", HeapWorkBudget::default())
        .expect("Private value")
        .id;
    let mut objects = ObjectHeap::new(&strings, ObjectHeapBudget::default()).expect("Objects");
    let root = objects
        .allocate(HeapWorkBudget::default())
        .expect("Root")
        .id;
    for (key, value) in [
        (b"privatefield".as_slice(), KernelValue::String(held)),
        (
            b"trace".as_slice(),
            KernelValue::Number(LegacyNumber::Integer(0)),
        ),
    ] {
        objects
            .put(root, key, value, &strings, HeapWorkBudget::default())
            .expect("Host field");
    }
    let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
        .with_objects(&mut objects, &[]);
    let error = module
        .program()
        .execute_with_heap(
            0,
            &[KernelValue::Object(root)],
            KernelCallBudget::default(),
            &mut context,
        )
        .expect_err("Mixed addition");
    assert_eq!(error.cause.kind, KernelErrorKind::UnmatchingTypes);
    let span = module
        .instruction_span(
            error.function.ok_or("Function")?,
            error.cause.instruction.ok_or("Instruction")?,
        )
        .ok_or("Source map")?;
    assert_eq!(
        &source
            [usize::try_from(span.start).expect("Start")..usize::try_from(span.end).expect("End")],
        b"+="
    );
    assert!(!format!("{error:?} {module:?} {context:?}").contains("Private"));
    assert_eq!(
        objects
            .get(root, b"trace", &strings, HeapWorkBudget::default())
            .expect("Only read receiver and RHS ran")
            .value,
        Some(KernelValue::Number(LegacyNumber::Integer(12)))
    );
    assert_eq!(
        objects
            .get(root, b"privatefield", &strings, HeapWorkBudget::default())
            .expect("Original target")
            .value,
        Some(KernelValue::String(held))
    );
    Ok(())
}

#[test]
fn noninteger_postfix_errors_preserve_values_and_absent_fields_without_private_payloads()
-> Result<(), String> {
    let source = b"main(s){s.PrivateField++;}";
    let module = module_compile_support::compile(
        source,
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Runtime postfix checks");
    for category in 0..5 {
        let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
        let held = strings
            .allocate(b"PrivatePayload", HeapWorkBudget::default())
            .expect("String")
            .id;
        let mut objects = ObjectHeap::new(&strings, ObjectHeapBudget::default()).expect("Objects");
        let root = objects
            .allocate(HeapWorkBudget::default())
            .expect("Root")
            .id;
        let value = match category {
            0 => None,
            1 => Some(KernelValue::Undefined),
            2 => Some(KernelValue::Number(LegacyNumber::Float(1.5))),
            3 => Some(KernelValue::String(held)),
            _ => Some(KernelValue::Object(root)),
        };
        if let Some(value) = value {
            objects
                .put(
                    root,
                    b"privatefield",
                    value,
                    &strings,
                    HeapWorkBudget::default(),
                )
                .expect("Existing value");
        }
        let before = objects.usage();
        let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
            .with_objects(&mut objects, &[]);
        let error = module
            .program()
            .execute_with_heap(
                0,
                &[KernelValue::Object(root)],
                KernelCallBudget::default(),
                &mut context,
            )
            .expect_err("Integer-only update");
        assert_eq!(
            error.cause.kind,
            if category == 2 {
                KernelErrorKind::Numeric(LegacyNumberError::ExpectedInteger)
            } else {
                KernelErrorKind::ExpectedNumber
            }
        );
        let span = module
            .instruction_span(
                error.function.ok_or("Function")?,
                error.cause.instruction.ok_or("Instruction")?,
            )
            .ok_or("Source map")?;
        assert_eq!(
            &source[usize::try_from(span.start).expect("Start")
                ..usize::try_from(span.end).expect("End")],
            b"++"
        );
        assert!(!format!("{error:?} {module:?} {context:?}").contains("Private"));
        assert_eq!(objects.usage(), before);
        assert_eq!(
            objects
                .get(root, b"privatefield", &strings, HeapWorkBudget::default())
                .expect("Unchanged selected target")
                .value,
            value
        );
    }
    Ok(())
}
