// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests compile raw localized data without claiming catalog translation/game services."
)]
use crate::module_compile_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::{FunctionCompileErrorKind, ModuleCompilationBudget};
use cod4x_script::execution::{KernelCallBudget, KernelErrorKind, KernelHeapContext, KernelValue};
use cod4x_script::heap::{HeapWorkBudget, StringHeap, StringHeapBudget};
use cod4x_script::legacy::LegacyNumber;
use cod4x_script::literal::{LegacyLiteralBudget, LegacyLiteralError, LegacyLiteralResource};

#[test]
fn localized_literal_bytes_and_tags_survive_source_drop_indirect_calls_and_returns()
-> Result<(), String> {
    let mut text = br#"main(){target=::copy;return [[target]](&"MENU_PRIVATE\n\"");}copy(input){value=input;return value;}"#.to_vec();
    let module = module_compile_support::compile(
        &text,
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Localized source");
    text.fill(0);
    let mut heap = StringHeap::new(StringHeapBudget::default()).expect("Heap");
    let mut context = KernelHeapContext::new(&mut heap, &[], HeapWorkBudget::default());
    let result = module
        .program()
        .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
        .expect("Localized copy");
    let KernelValue::LocalizedString(id) = result.value else {
        return Err("Localized tag lost".into());
    };
    assert_eq!(
        context.get(id).expect("Owned raw localization key"),
        b"MENU_PRIVATE\n\""
    );
    assert_eq!(module.usage().string_literals, 1);
    assert_eq!(context.heap_usage().live_strings, 1);
    assert!(!format!("{result:?} {module:?} {context:?}").contains("MENU_PRIVATE"));
    Ok(())
}

#[test]
fn localized_source_equality_uses_contents_and_isstring_preserves_native_plain_only_tag() {
    let text = br#"main(){a=&"MENU";b=&"MENU";if(a!=b){return 99;}if(a==&"OTHER"){return 99;}return isstring(a)+isstring("MENU");}"#;
    let module = module_compile_support::compile(
        text,
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Localized equality");
    let mut heap = StringHeap::new(StringHeapBudget::default()).expect("Heap");
    let mut context = KernelHeapContext::new(&mut heap, &[], HeapWorkBudget::default());
    assert_eq!(
        module
            .program()
            .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
            .expect("Content equality/distinct tags")
            .value,
        KernelValue::Number(LegacyNumber::Integer(1))
    );
}

#[test]
fn localized_sources_keep_typed_unsupported_mixed_arithmetic_and_bool_errors_located()
-> Result<(), String> {
    for (text, kind) in [
        (
            br#"main(){return &"PrivateKey"+"plain";}"#.as_slice(),
            KernelErrorKind::UnmatchingTypes,
        ),
        (
            br#"main(){return &"PrivateKey"=="plain";}"#,
            KernelErrorKind::UnmatchingTypes,
        ),
        (
            br#"main(){if(&"PrivateKey"){return 1;}return 0;}"#,
            KernelErrorKind::ExpectedNumber,
        ),
    ] {
        let module = module_compile_support::compile(
            text,
            LegacyDeveloperMode::Disabled,
            ModuleCompilationBudget::default(),
        )
        .expect("Located localized operation");
        let mut heap = StringHeap::new(StringHeapBudget::default()).expect("Heap");
        let mut context = KernelHeapContext::new(&mut heap, &[], HeapWorkBudget::default());
        let error = module
            .program()
            .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
            .expect_err("No implicit translation/coercion");
        assert_eq!(error.cause.kind, kind);
        assert!(
            module
                .instruction_span(
                    error.function.ok_or("Function")?,
                    error.cause.instruction.ok_or("Instruction")?
                )
                .is_some()
        );
        assert!(!format!("{error:?} {context:?}").contains("PrivateKey"));
    }
    Ok(())
}

#[test]
fn localized_prefix_and_decoded_bytes_use_exact_per_token_limits() {
    let text = br#"main(){return &"\n\n";}"#;
    let mut exact = ModuleCompilationBudget::default();
    exact.function.literal = LegacyLiteralBudget {
        max_encoded_bytes: 7,
        max_string_bytes: 2,
    };
    let module = module_compile_support::compile(text, LegacyDeveloperMode::Disabled, exact)
        .expect("Exact localized prefix/payload");
    assert_eq!(
        module.usage().staged_string_bytes,
        size_of::<Vec<u8>>() as u64 + 2
    );
    for (encoded, decoded, resource) in [
        (6, 2, LegacyLiteralResource::EncodedBytes),
        (7, 1, LegacyLiteralResource::StringBytes),
    ] {
        let mut budget = exact;
        budget.function.literal = LegacyLiteralBudget {
            max_encoded_bytes: encoded,
            max_string_bytes: decoded,
        };
        assert_eq!(
            module_compile_support::compile(text, LegacyDeveloperMode::Disabled, budget)
                .expect_err("One less localized token resource")
                .cause
                .kind,
            FunctionCompileErrorKind::Literal(LegacyLiteralError::BudgetExceeded(resource))
        );
    }
}
