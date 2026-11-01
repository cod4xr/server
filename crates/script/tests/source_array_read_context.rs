// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests explicit package contexts and hostile identities without publishing borrowed owners or private bytes."
)]
use crate::array_read_support::{self, number};
use crate::module_compile_support;
use crate::package_compile_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::{ModuleCompilationBudget, ScriptPackageBudget};
use cod4x_script::execution::{KernelCallBudget, KernelErrorKind, KernelValue};
use cod4x_script::heap::{ArrayKey, HeapWorkBudget, ObjectHeapErrorKind};

#[test]
fn package_imports_and_qualified_indirect_calls_read_the_same_host_data() {
    let package = package_compile_support::compile(
        &[
            (
                b"root",
                b"#include lib; main(){f=lib::pick;return read(game)+[[f]](game);}",
            ),
            (
                b"lib",
                b"pick(a){return read(a);}read(a){return a[0]+a.size;}",
            ),
        ],
        LegacyDeveloperMode::Disabled,
        ScriptPackageBudget::default(),
    )
    .expect("Read package");
    let mut state = array_read_support::state(0, 2);
    let entry = package.function_for_declaration(0, 0).expect("Entry");
    let result = package
        .program()
        .execute_with_heap(
            entry,
            &[],
            KernelCallBudget::default(),
            &mut state.context(HeapWorkBudget::default()),
        )
        .expect("Cross-file reads");
    assert_eq!(result.value, number(32));
    state.assert_host_owners(2);
}

#[test]
fn foreign_and_stale_array_and_key_arguments_reject_without_changing_host_ownership() {
    let module = module_compile_support::compile(
        b"main(a,key){return a[key];}",
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Read source");
    let work = HeapWorkBudget::default();
    let mut state = array_read_support::state(0, 1);
    let other = array_read_support::state(0, 1);
    let dead = state
        .objects
        .copy_array_value(state.array, work)
        .expect("Temporary owner")
        .value;
    state
        .objects
        .release_array_value(dead, work)
        .expect("Stale location");
    let stale_key = state
        .strings
        .allocate(b"PrivateDeadKey", work)
        .expect("Key")
        .id;
    state
        .strings
        .reclaim(&[state.plain, state.localized], work)
        .expect("Retain payloads only");
    for (array, key, expected) in [
        (
            other.array,
            number(0),
            KernelErrorKind::ObjectHeap(ObjectHeapErrorKind::ForeignArrayValue),
        ),
        (
            dead,
            number(0),
            KernelErrorKind::ObjectHeap(ObjectHeapErrorKind::StaleArrayValue),
        ),
        (
            state.array,
            KernelValue::String(other.plain),
            KernelErrorKind::ForeignStringReference,
        ),
        (
            state.array,
            KernelValue::String(stale_key),
            KernelErrorKind::StaleStringReference,
        ),
    ] {
        let error = module
            .program()
            .execute_with_heap(
                0,
                &[KernelValue::Array(array), key],
                KernelCallBudget::default(),
                &mut state.context(work),
            )
            .expect_err("Hostile argument");
        assert_eq!(error.cause.kind, expected);
        assert!(!format!("{error:?} {error}").contains("PrivateDeadKey"));
        state.assert_host_owners(1);
    }
    state
        .strings
        .reclaim(&[], work)
        .expect("Host invalidates stored string payloads");
    let host = state.array;
    let error = module
        .program()
        .execute_with_heap(
            0,
            &[KernelValue::Array(host), number(0x7f_ffff)],
            KernelCallBudget::default(),
            &mut state.context(work),
        )
        .expect_err("Stale stored result");
    assert_eq!(error.cause.kind, KernelErrorKind::StaleStringReference);
    state.assert_host_owners(1);
}

#[test]
fn function_read_preserves_image_identity_and_indirect_call_rejects_another_image() {
    let source = b"main(){return [[game[\"callback\"]]]();}callback(){return 7;}";
    let module = module_compile_support::compile(
        source,
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Consumer image");
    let other = module_compile_support::compile(
        source,
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Distinct immutable image");
    let mut state = array_read_support::state(0, 1);
    let work = HeapWorkBudget::default();
    state
        .objects
        .array_put_unique(
            state.array,
            ArrayKey::Bytes(b"callback"),
            KernelValue::Function(
                other
                    .program()
                    .function_reference(1)
                    .expect("Foreign image token"),
            ),
            &state.strings,
            work,
        )
        .expect("Opaque checked payload");
    let error = module
        .program()
        .execute_with_heap(
            0,
            &[],
            KernelCallBudget::default(),
            &mut state.context(work),
        )
        .expect_err("No token reinterpretation through read");
    assert_eq!(error.cause.kind, KernelErrorKind::ForeignFunctionReference);
    assert_eq!(
        state
            .objects
            .array_owner_count(state.data, work)
            .expect("Host retained")
            .value,
        1
    );
}
