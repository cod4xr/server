// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests check real source maps and explicitly unsupported string profiles."
)]
use crate::module_compile_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::{FunctionCompileErrorKind, ModuleCompilationBudget};
use cod4x_script::execution::{
    KernelCallBudget, KernelErrorKind, KernelHeapContext, KernelResource, KernelValue,
};
use cod4x_script::heap::{HeapWorkBudget, StringHeap, StringHeapBudget};

#[test]
fn source_literal_and_operator_failures_preserve_exact_spans_without_private_payload()
-> Result<(), String> {
    let text = br#"main(){return "PrivatePayload"+1;}"#;
    let module = module_compile_support::compile(
        text,
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Owned source maps");
    for (maximum, expected, token) in [
        (100, KernelErrorKind::UnmatchingTypes, b"+".as_slice()),
        (
            4,
            KernelErrorKind::BudgetExceeded {
                resource: KernelResource::SingleStringBytes,
                requested: 14,
                maximum: 4,
            },
            b"\"PrivatePayload\"".as_slice(),
        ),
    ] {
        let mut heap = StringHeap::new(StringHeapBudget {
            max_single_string_bytes: maximum,
            ..StringHeapBudget::default()
        })
        .expect("Heap");
        let mut context = KernelHeapContext::new(&mut heap, &[], HeapWorkBudget::default());
        let error = module
            .program()
            .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
            .expect_err("Located source operation");
        assert_eq!(error.cause.kind, expected);
        let span = module
            .instruction_span(
                error.function.ok_or("Missing function")?,
                error.cause.instruction.ok_or("Missing instruction")?,
            )
            .ok_or("Missing map")?;
        assert_eq!(
            &text[usize::try_from(span.start).expect("Offset")
                ..usize::try_from(span.end).expect("Offset")],
            token
        );
        assert!(!format!("{error:?} {module:?} {context:?}").contains("PrivatePayload"));
    }
    Ok(())
}

#[test]
fn unsupported_localized_field_body_prevents_publication_and_developer_tables_stay_scoped()
-> Result<(), String> {
    let text = br#"main(){return "ordinary";}/# hidden(){return &"PrivateLocalized".missing;}#/"#;
    let module = module_compile_support::compile(
        text,
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Disabled localized body");
    assert_eq!(module.usage().string_literals, 1);
    let error = module_compile_support::compile(
        text,
        LegacyDeveloperMode::Enabled,
        ModuleCompilationBudget::default(),
    )
    .expect_err("Complete publication rejects active unsupported field access");
    assert_eq!(error.declaration_index(), Some(1));
    assert_eq!(
        error.cause.kind,
        FunctionCompileErrorKind::UnsupportedPrimary
    );
    assert!(!format!("{error:?}").contains("PrivateLocalized"));
    let text = br#"main(){return "ordinary";}/# hidden(){return "developer";}#/"#;
    let module = module_compile_support::compile(
        text,
        LegacyDeveloperMode::Enabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Active plain developer literal table");
    assert_eq!(module.usage().string_literals, 2);
    let mut heap = StringHeap::new(StringHeapBudget::default()).expect("Heap");
    let mut context = KernelHeapContext::new(&mut heap, &[], HeapWorkBudget::default());
    let result = module
        .program()
        .execute_with_heap(
            module.function_for_declaration(1).expect("Developer entry"),
            &[],
            KernelCallBudget::default(),
            &mut context,
        )
        .expect("Explicit developer invocation");
    let KernelValue::String(id) = result.value else {
        return Err("Expected developer string".into());
    };
    assert_eq!(context.get(id).expect("Developer bytes"), b"developer");
    Ok(())
}
