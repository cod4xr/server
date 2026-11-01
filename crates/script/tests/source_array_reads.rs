// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests execute bounded host-origin reads through compiled source and checked host values."
)]
use crate::array_read_support::{self, number};
use crate::module_compile_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::ModuleCompilationBudget;
use cod4x_script::execution::{KernelCallBudget, KernelErrorKind, KernelValue};
use cod4x_script::heap::{ArrayKey, HeapWorkBudget};
use cod4x_script::legacy::{LegacyNumber, LegacyVector};

#[test]
fn source_reads_distinguish_key_bytes_and_preserve_missing_undefined_and_shared_identity() {
    let module = module_compile_support::compile(
        b"main(a,key){o=a[\"Case\"];o.changed=31;
          return a.size+a[\"case\"]+a[\"Case\"].changed+a[key]
          +isdefined(a[1])+isdefined(a[65535]);}",
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Source reads and ordinary shared mutation");
    let mut state = array_read_support::state(3, 1);
    let key = state.key_value(2, 65535).expect("Host string key");
    let keys = state.keys();
    assert_eq!(state.strings.get(state.plain), Ok(b"Payload101".as_slice()));
    let host = state.array;
    let result = module
        .program()
        .execute_with_heap(
            0,
            &[KernelValue::Array(host), key],
            KernelCallBudget::default(),
            &mut state.context(HeapWorkBudget::default()),
        )
        .expect("Read through borrowed argument");
    assert_eq!(result.value, number(9 + 13 + 31 - 123));
    assert_eq!(state.keys(), keys);
    assert_eq!(state.objects.field_count(state.shared), Ok(1));
    assert_eq!(
        state
            .objects
            .array_owner_count(state.data, HeapWorkBudget::default())
            .expect("Only host owner")
            .value,
        1
    );
}

#[test]
fn source_index_evaluates_key_before_receiver_and_plain_calls_share_game() {
    let module = module_compile_support::compile(
        b"main(){level.order=0;v=receiver()[index()];return level.order*10+v;}
          receiver(){level.order=level.order*10+2;return game;}
          index(){level.order=level.order*10+1;return 0;}",
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Observable evaluation order");
    let mut state = array_read_support::state(0, 1);
    let result = module
        .program()
        .execute_with_heap(
            0,
            &[],
            KernelCallBudget::default(),
            &mut state.context(HeapWorkBudget::default()),
        )
        .expect("Read returned host-derived alias");
    assert_eq!(result.value, number(127));
    assert_eq!(
        state
            .objects
            .array_owner_count(state.data, HeapWorkBudget::default())
            .expect("Host survives two calls")
            .value,
        1
    );
}

#[test]
fn checked_function_and_inline_vector_payloads_compose_with_source_reads() {
    let module = module_compile_support::compile(
        b"main(){return [[game[\"callback\"]]](7)+game[\"vector\"][1];}
          callback(x){return x+2;}",
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Read checked Rust values; no native pointer ABI claim");
    let mut state = array_read_support::state(0, 1);
    let callback = module
        .program()
        .function_reference(1)
        .expect("Same image function");
    let work = HeapWorkBudget::default();
    state
        .objects
        .array_put_unique(
            state.array,
            ArrayKey::Bytes(b"callback"),
            KernelValue::Function(callback),
            &state.strings,
            work,
        )
        .expect("Checked function entry");
    state
        .objects
        .array_put_unique(
            state.array,
            ArrayKey::Bytes(b"vector"),
            KernelValue::Vector(LegacyVector::from_numbers([
                LegacyNumber::Integer(2),
                LegacyNumber::Integer(3),
                LegacyNumber::Integer(5),
            ])),
            &state.strings,
            work,
        )
        .expect("Inline vector entry");
    let result = module
        .program()
        .execute_with_heap(
            0,
            &[],
            KernelCallBudget::default(),
            &mut state.context(work),
        )
        .expect("Source composition");
    assert_eq!(result.value, KernelValue::Number(LegacyNumber::Float(12.0)));
    assert_eq!(
        state
            .objects
            .array_owner_count(state.data, work)
            .expect("Host owner")
            .value,
        1
    );
}

#[test]
fn invalid_source_keys_cleanup_all_aliases_and_the_same_context_can_execute_again() {
    let module = module_compile_support::compile(
        b"main(key){a=game;return relay(a,key);}relay(a,key){return a[key];}
          good(){return game.size;}",
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Failures inside a callee");
    let mut state = array_read_support::state(0, 7);
    let localized = state.localized;
    let array_key = state.array;
    let mut context = state.context(HeapWorkBudget::default());
    for (key, kind) in [
        (number(-1), KernelErrorKind::ArrayIndexOutOfRange),
        (number(0x80_0000), KernelErrorKind::ArrayIndexOutOfRange),
        (KernelValue::Undefined, KernelErrorKind::ExpectedArrayIndex),
        (
            KernelValue::LocalizedString(localized),
            KernelErrorKind::ExpectedArrayIndex,
        ),
        (
            KernelValue::Array(array_key),
            KernelErrorKind::ExpectedArrayIndex,
        ),
        (
            KernelValue::Number(LegacyNumber::Float(f32::NAN)),
            KernelErrorKind::ExpectedArrayIndex,
        ),
    ] {
        let error = module
            .program()
            .execute_with_heap(0, &[key], KernelCallBudget::default(), &mut context)
            .expect_err("Invalid array index");
        assert_eq!(error.cause.kind, kind);
        assert_eq!(
            context
                .object_heap_usage()
                .expect("Objects")
                .live_array_values,
            8
        );
        assert_eq!(
            module
                .program()
                .execute_with_heap(2, &[], KernelCallBudget::default(), &mut context,)
                .expect("Recovery")
                .value,
            number(9)
        );
    }
    drop(context);
    state.assert_host_owners(7);
}
