// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests measure source fields, exact table staging/proof admission and located errors."
)]
use crate::module_compile_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::ModuleCompilationBudget;
use cod4x_script::execution::{KernelCallBudget, KernelErrorKind, KernelHeapContext};
use cod4x_script::heap::{
    HeapWorkBudget, ObjectHeap, ObjectHeapBudget, StringHeap, StringHeapBudget,
};

#[test]
fn mixed_field_and_literal_tables_charge_exact_global_compiler_and_proof_budgets() {
    let source = b"main(){o=spawnstruct();o.private=\"payload\";return helper(o);}helper(o){o.second=1;return o.private.size;}";
    let usage = module_compile_support::compile(
        source,
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Measured mixed tables")
    .usage();
    assert_eq!((usage.field_names, usage.string_literals), (3, 1));
    assert_eq!(
        usage.staged_field_bytes,
        3 * size_of::<Vec<u8>>() as u64 + 20
    );
    let mut exact = ModuleCompilationBudget {
        max_steps: usage.compiler_steps,
        max_metadata_bytes: usage.peak_metadata_bytes,
        ..ModuleCompilationBudget::default()
    };
    exact.verification.max_steps = usage.verification.verification_steps;
    exact.verification.max_metadata_bytes = usage.verification.verification_metadata_bytes;
    assert_eq!(
        module_compile_support::compile(source, LegacyDeveloperMode::Disabled, exact)
            .expect("Exact all scopes")
            .usage(),
        usage
    );
    for resource in 0..4 {
        let mut budget = exact;
        match resource {
            0 => budget.max_steps -= 1,
            1 => budget.max_metadata_bytes -= 1,
            2 => budget.verification.max_steps -= 1,
            _ => budget.verification.max_metadata_bytes -= 1,
        }
        assert!(
            module_compile_support::compile(source, LegacyDeveloperMode::Disabled, budget).is_err()
        );
    }
}

#[test]
fn invalid_field_receivers_do_not_autovivify_and_errors_retain_private_source_spans()
-> Result<(), String> {
    for (source, token) in [
        (
            b"main(){o=undefined;return o.PrivateField;}".as_slice(),
            b".".as_slice(),
        ),
        (
            b"main(){o=undefined;o.PrivateField=1;}".as_slice(),
            b"=".as_slice(),
        ),
        (
            b"main(){o=undefined;o.PrivateField=undefined;}".as_slice(),
            b"=".as_slice(),
        ),
        (
            b"main(){return \"PrivatePayload\".PrivateField;}".as_slice(),
            b".".as_slice(),
        ),
    ] {
        let module = module_compile_support::compile(
            source,
            LegacyDeveloperMode::Disabled,
            ModuleCompilationBudget::default(),
        )
        .expect("Receiver is checked at execution");
        let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
        let mut objects = ObjectHeap::new(&strings, ObjectHeapBudget::default()).expect("Objects");
        let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
            .with_objects(&mut objects, &[]);
        let error = module
            .program()
            .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
            .expect_err("No implicit object creation");
        assert_eq!(error.cause.kind, KernelErrorKind::ExpectedFieldObject);
        let span = module
            .instruction_span(
                error.function.ok_or("Missing function")?,
                error.cause.instruction.ok_or("Missing instruction")?,
            )
            .ok_or("Missing source span")?;
        assert_eq!(
            &source[usize::try_from(span.start).expect("Start")
                ..usize::try_from(span.end).expect("End")],
            token
        );
        assert_eq!(context.object_heap_usage().expect("Usage").live_objects, 0);
        assert!(!format!("{module:?} {error:?} {context:?}").contains("Private"));
    }
    for source in [
        b"main(){o=spawnstruct();o.field=1;o.field+=1;return o.field;}".as_slice(),
        b"main(){o=spawnstruct();o.field=1;o.field++;return o.field;}".as_slice(),
    ] {
        assert!(
            module_compile_support::compile(
                source,
                LegacyDeveloperMode::Disabled,
                ModuleCompilationBudget::default()
            )
            .is_err()
        );
    }
    Ok(())
}
