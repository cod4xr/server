// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests compile real plain-string builtin arguments and explicit invocation failures."
)]
use crate::module_compile_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::{FunctionCompileErrorKind, ModuleCompilationBudget};
use cod4x_script::execution::{KernelCallBudget, KernelErrorKind, KernelHeapContext, KernelValue};
use cod4x_script::heap::{HeapWorkBudget, StringHeap, StringHeapBudget};
use cod4x_script::legacy::LegacyNumber;

#[test]
fn isstring_accepts_plain_source_values_and_preserves_case_identity_and_other_type_tags() {
    for (source, expected) in [
        (br#"main(){return IsStRiNg("")+isstring("x")+isstring(0)+isstring(0.0)+isstring(undefined)+isstring((1,2,3))+isstring(::main);}"#.as_slice(), 2),
        (br#"main(){value=join("a","b");return check(value);}join(a,b){return a+b;}check(value){return isstring(value);}"#.as_slice(), 1),
    ] {
        let module = module_compile_support::compile(source, LegacyDeveloperMode::Disabled, ModuleCompilationBudget::default()).expect("String predicates");
        let mut heap = StringHeap::new(StringHeapBudget::default()).expect("Heap");
        let mut context = KernelHeapContext::new(&mut heap, &[], HeapWorkBudget::default());
        assert_eq!(module.program().execute_with_heap(0, &[], KernelCallBudget::default(), &mut context).expect("Runtime type predicate").value, KernelValue::Number(LegacyNumber::Integer(expected)));
    }
}

#[test]
fn local_and_developer_string_named_functions_keep_precedence_over_builtin_fallback() {
    for source in [
        br#"main(){return IsStRiNg("x");}isstring(value){return 17;}"#.as_slice(),
        br#"main(){return [[::isstring]]("x");}isstring(value){return 17;}"#,
    ] {
        let module = module_compile_support::compile(
            source,
            LegacyDeveloperMode::Disabled,
            ModuleCompilationBudget::default(),
        )
        .expect("Local target shadows builtin");
        let mut heap = StringHeap::new(StringHeapBudget::default()).expect("Heap");
        let mut context = KernelHeapContext::new(&mut heap, &[], HeapWorkBudget::default());
        assert_eq!(
            module
                .program()
                .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
                .expect("Local call")
                .value,
            KernelValue::Number(LegacyNumber::Integer(17))
        );
    }
    let source = br#"main(){return isstring("x");}/# isstring(value){return 17;}#/"#;
    let module = module_compile_support::compile(
        source,
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Inactive target does not shadow");
    let mut heap = StringHeap::new(StringHeapBudget::default()).expect("Heap");
    let mut context = KernelHeapContext::new(&mut heap, &[], HeapWorkBudget::default());
    assert_eq!(
        module
            .program()
            .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
            .expect("Builtin fallback")
            .value,
        KernelValue::Number(LegacyNumber::Integer(1))
    );
    assert_eq!(
        module_compile_support::compile(
            source,
            LegacyDeveloperMode::Enabled,
            ModuleCompilationBudget::default()
        )
        .expect_err("Developer visibility cannot fall back")
        .cause
        .kind,
        FunctionCompileErrorKind::DeveloperFunctionReference
    );
}

#[test]
fn string_predicate_arity_runs_only_after_selected_arguments_and_remains_located()
-> Result<(), String> {
    let source = br#"main(flag){if(flag){return isstring("a","b");}return 7;}"#;
    let module = module_compile_support::compile(
        source,
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Skipped bad arity is compilable");
    let mut heap = StringHeap::new(StringHeapBudget::default()).expect("Heap");
    let mut context = KernelHeapContext::new(&mut heap, &[], HeapWorkBudget::default());
    assert_eq!(
        module
            .program()
            .execute_with_heap(
                0,
                &[KernelValue::Number(LegacyNumber::Integer(0))],
                KernelCallBudget::default(),
                &mut context
            )
            .expect("Skipped call")
            .value,
        KernelValue::Number(LegacyNumber::Integer(7))
    );
    assert_eq!(context.heap_usage().live_strings, 0);
    let error = module
        .program()
        .execute_with_heap(
            0,
            &[KernelValue::Number(LegacyNumber::Integer(1))],
            KernelCallBudget::default(),
            &mut context,
        )
        .expect_err("Selected bad arity");
    assert_eq!(
        error.cause.kind,
        KernelErrorKind::ArgumentCount {
            expected: 1,
            actual: 2
        }
    );
    assert_eq!(context.heap_usage().live_strings, 2);
    let span = module
        .instruction_span(
            error.function.ok_or("Function")?,
            error.cause.instruction.ok_or("Instruction")?,
        )
        .ok_or("Map")?;
    assert_eq!(
        &source[usize::try_from(span.start).expect("Offset")
            ..usize::try_from(span.end).expect("Offset")],
        br#"isstring("a","b")"#
    );
    Ok(())
}
