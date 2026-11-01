// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Real source RHS graphs must stay rooted through COW and insertion retries."
)]
use crate::{
    array_read_support::number, source_array_write_support::State, source_scope_support::compile,
};
use cod4x_script::compilation::CompiledModule;
use cod4x_script::execution::{KernelCallBudget, KernelHeapUsage, KernelModuleError, KernelValue};
use cod4x_script::heap::{ArrayKey, HeapErrorKind, HeapWorkBudget, ObjectHeapBudget};

fn run(
    module: &CompiledModule,
    key: u64,
    budget: HeapWorkBudget,
) -> (Result<(), KernelModuleError>, KernelHeapUsage) {
    let work = HeapWorkBudget::default();
    let mut state = State::new(ObjectHeapBudget {
        max_live_fields: if key == 0 { 3 } else { 4 },
        ..ObjectHeapBudget::default()
    });
    let garbage_string = state
        .strings
        .allocate(b"unreachable", work)
        .expect("Garbage")
        .id;
    let host = state.objects.new_array_value(work).expect("Host").value;
    let host_data = state
        .objects
        .array_data(host, work)
        .expect("Host data")
        .value;
    state
        .objects
        .array_put_unique(host, ArrayKey::Index(0), number(7), &state.strings, work)
        .expect("Host field");
    let garbage = state.objects.allocate(work).expect("Garbage object").id;
    state
        .objects
        .put(
            garbage,
            b"discard",
            KernelValue::String(garbage_string),
            &state.strings,
            work,
        )
        .expect("Garbage field");
    let (result, usage) = state.run(
        module,
        &[KernelValue::Array(host)],
        KernelCallBudget::default(),
        budget,
    );
    assert_eq!(
        state.objects.usage().live_array_values,
        1 + u64::from(result.is_ok())
    );
    let result = result.map(|result| {
        let returned = match result.value {
            KernelValue::Array(id) => Some(id),
            _ => None,
        }
        .expect("Owned result");
        let data = state
            .objects
            .array_data(returned, work)
            .expect("Detached data")
            .value;
        assert_ne!(host_data, data);
        assert_payload(&state, data, key);
        assert_eq!(
            state.strings.get(garbage_string),
            Err(HeapErrorKind::StaleReference)
        );
        assert_eq!((usage.object_collections, usage.collections), (1, 1));
        state
            .objects
            .release_array_value(returned, work)
            .expect("Host releases result");
    });
    assert_eq!(state.objects.usage().live_array_values, 1);
    assert_eq!(
        state
            .objects
            .array_owner_count(host_data, work)
            .expect("Host only")
            .value,
        1
    );
    assert_eq!(
        state
            .objects
            .array_get(host_data, ArrayKey::Index(0), &state.strings, work)
            .expect("Host payload unchanged")
            .value,
        Some(number(7))
    );
    (result, usage)
}

fn assert_payload(state: &State, data: cod4x_script::heap::ArrayId, key: u64) {
    let work = HeapWorkBudget::default();
    let object = match state
        .objects
        .array_get(data, ArrayKey::Index(key), &state.strings, work)
        .expect("Stored RHS")
        .value
    {
        Some(KernelValue::Object(id)) => Some(id),
        _ => None,
    }
    .expect("Runtime object");
    let string = match state
        .objects
        .get(object, b"payload", &state.strings, work)
        .expect("RHS graph")
        .value
    {
        Some(KernelValue::String(id)) => Some(id),
        _ => None,
    }
    .expect("Runtime string");
    assert_eq!(state.strings.get(string), Ok(b"leftright".as_slice()));
}

#[test]
fn source_rhs_objects_and_runtime_strings_survive_each_cow_or_insertion_pressure_prefix() {
    for key in [0, 1] {
        let source = format!(
            "main(host){{host[{key}]=make();return host;}}
            make(){{o=spawnstruct();o.payload=\"left\"+\"right\";return o;}}"
        );
        let module = compile(source.as_bytes());
        let (result, usage) = run(&module, key, HeapWorkBudget::default());
        result.expect("Paired pressure retry");
        let exact = HeapWorkBudget {
            max_steps: usage.steps,
            max_scratch_bytes: usage.peak_scratch_bytes,
        };
        run(&module, key, exact).0.expect("Exact combined scopes");
        for maximum in 0..usage.steps {
            run(
                &module,
                key,
                HeapWorkBudget {
                    max_steps: maximum,
                    ..exact
                },
            )
            .0
            .expect_err("Every shorter COW/insertion/RHS cleanup prefix");
        }
        run(
            &module,
            key,
            HeapWorkBudget {
                max_scratch_bytes: exact.max_scratch_bytes - 1,
                ..exact
            },
        )
        .0
        .expect_err("One-less combined scratch");
    }
}
