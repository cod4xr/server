// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests compile finite real primitive definedness modules."
)]
use crate::module_compile_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::{FunctionCompileErrorKind, ModuleCompilationBudget};
use cod4x_script::execution::{KernelCallBudget, KernelErrorKind, KernelHeapContext, KernelValue};
use cod4x_script::heap::{HeapWorkBudget, StringHeap, StringHeapBudget};
use cod4x_script::legacy::LegacyNumber;

#[test]
fn definedness_distinguishes_missing_parameter_values_and_accepts_evaluated_extras() {
    let text = br#"main(){if(isdefined(optional())){return 99;}return probe()+isdefined(0,undefined)+isdefined("")+isdefined(&"")+isdefined((0,0,0))+isdefined(::probe);}optional(x){return x;}probe(x){if(isdefined(x)){return 99;}return 2;}"#;
    let module = module_compile_support::compile(
        text,
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Optional values and extras");
    let mut heap = StringHeap::new(StringHeapBudget::default()).expect("Heap");
    let mut context = KernelHeapContext::new(&mut heap, &[], HeapWorkBudget::default());
    assert_eq!(
        module
            .program()
            .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
            .expect("Definedness is not truthiness")
            .value,
        KernelValue::Number(LegacyNumber::Integer(7))
    );
}

#[test]
fn definedness_checks_arity_only_on_executed_calls_and_evaluates_ignored_arguments_first()
-> Result<(), String> {
    for (text, function, token, missing) in [
        (
            br"main(){return isdefined();}".as_slice(),
            0,
            b"isdefined()".as_slice(),
            true,
        ),
        (
            br"main(){return isdefined(undefined,bad());}bad(){return 1/0;}",
            1,
            b"/",
            false,
        ),
    ] {
        let module = module_compile_support::compile(
            text,
            LegacyDeveloperMode::Disabled,
            ModuleCompilationBudget::default(),
        )
        .expect("Runtime arity proof");
        let error = module
            .program()
            .execute(0, &[], KernelCallBudget::default())
            .expect_err("Executed failure");
        if missing {
            assert_eq!(
                error.cause.kind,
                KernelErrorKind::ArgumentCount {
                    expected: 1,
                    actual: 0
                }
            );
        } else {
            assert!(matches!(error.cause.kind, KernelErrorKind::Numeric(_)));
        }
        assert_eq!(error.function, Some(function));
        let span = module
            .instruction_span(function, error.cause.instruction.ok_or("Missing PC")?)
            .ok_or("Missing map")?;
        assert_eq!(
            &text[usize::try_from(span.start).expect("Offset")
                ..usize::try_from(span.end).expect("Offset")],
            token
        );
    }
    let module = module_compile_support::compile(
        br"main(){if(0){return isdefined();}return isdefined(undefined,1);}",
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Skipped missing argument call");
    assert_eq!(
        module
            .program()
            .execute(0, &[], KernelCallBudget::default())
            .expect("Skipped arity and Undefined first argument")
            .value,
        KernelValue::Number(LegacyNumber::Integer(0))
    );
    Ok(())
}

#[test]
fn definedness_fallback_keeps_local_shadowing_case_and_developer_policies() {
    let module = module_compile_support::compile(
        br"main(){return IsDefined(0);}isdefined(x){return x+2;}",
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Local shadow");
    assert_eq!(
        module
            .program()
            .execute(0, &[], KernelCallBudget::default())
            .expect("Local result")
            .value,
        KernelValue::Number(LegacyNumber::Integer(2))
    );
    let text = br"main(){return IsDefined(0);}/# isdefined(x){return x+2;}#/";
    let module = module_compile_support::compile(
        text,
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Disabled local leaves fallback");
    assert_eq!(
        module
            .program()
            .execute(0, &[], KernelCallBudget::default())
            .expect("Builtin result")
            .value,
        KernelValue::Number(LegacyNumber::Integer(1))
    );
    let error = module_compile_support::compile(
        text,
        LegacyDeveloperMode::Enabled,
        ModuleCompilationBudget::default(),
    )
    .expect_err("Active developer local cannot be bypassed");
    assert_eq!(
        error.cause.kind,
        FunctionCompileErrorKind::DeveloperFunctionReference
    );
}

#[test]
fn source_definedness_evaluates_all_extra_string_arguments_without_native_byte_count_limits() {
    let source = format!(
        "main(){{return isdefined(undefined,{});}}",
        vec!["\"x\""; 256].join(",")
    );
    let module = module_compile_support::compile(
        source.as_bytes(),
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Wide source call");
    let mut heap = StringHeap::new(StringHeapBudget::default()).expect("Heap");
    let mut context = KernelHeapContext::new(&mut heap, &[], HeapWorkBudget::default());
    let result = module
        .program()
        .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
        .expect("All extras evaluated then ignored");
    assert_eq!(result.value, KernelValue::Number(LegacyNumber::Integer(0)));
    assert_eq!(module.usage().string_literals, 256);
    assert_eq!(context.heap_usage().live_strings, 256);
    assert_eq!(context.heap_usage().payload_bytes, 256);
    assert_eq!(result.usage.peak_frames, 1);
}
