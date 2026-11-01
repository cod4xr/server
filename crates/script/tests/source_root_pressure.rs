// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests root-only ordinary graphs and game data through paired collection before any root read."
)]
use crate::module_compile_support;
use crate::script_root_support;
use cod4x_script::binding::LegacyDeveloperMode;
use cod4x_script::compilation::ModuleCompilationBudget;
use cod4x_script::execution::{KernelCallBudget, KernelHeapContext, KernelValue};
use cod4x_script::heap::{
    ArrayKey, HeapErrorKind, HeapWorkBudget, ObjectHeapBudget, ObjectHeapErrorKind, ObjectId,
    StringId,
};
use cod4x_script::legacy::LegacyNumber;

#[test]
fn root_only_struct_and_string_graphs_survive_collection_before_the_first_root_read() {
    let module = module_compile_support::compile(
        b"main(){temp=spawnstruct();return isdefined(level.payload)+isdefined(self.payload)
          +isdefined(anim.payload)+isdefined(game);}",
        LegacyDeveloperMode::Disabled,
        ModuleCompilationBudget::default(),
    )
    .expect("Pressure before root evaluation");
    let work = HeapWorkBudget::default();
    let mut state = script_root_support::state(ObjectHeapBudget {
        max_live_objects: 7,
        max_object_slots: 7,
        ..ObjectHeapBudget::default()
    });
    let child = state.objects.allocate(work).expect("Nested level child").id;
    let game_child = state.objects.allocate(work).expect("Shared game child").id;
    let garbage = state.objects.allocate(work).expect("Unreachable object").id;
    let object_roots = [
        state.level,
        state.receiver,
        state.animation,
        child,
        game_child,
    ];
    let held = attach_payloads(&mut state, &object_roots);
    state
        .objects
        .put(
            state.level,
            b"child",
            KernelValue::Object(child),
            &state.strings,
            work,
        )
        .expect("Deep root");
    state
        .objects
        .array_put_unique(
            state.game,
            ArrayKey::Index(0),
            KernelValue::Object(game_child),
            &state.strings,
            work,
        )
        .expect("Game graph");
    let discarded = state
        .strings
        .allocate(b"unreachable private text", work)
        .expect("Garbage string")
        .id;
    let roots = state.roots();
    let mut context = KernelHeapContext::new(&mut state.strings, &[], work)
        .with_objects(&mut state.objects, &[])
        .with_script_roots(roots);
    let result = module
        .program()
        .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)
        .expect("Root-only graphs must survive");
    assert_eq!(result.value, KernelValue::Number(LegacyNumber::Integer(4)));
    assert_eq!(
        (
            context.usage().object_collections,
            context.usage().collections
        ),
        (1, 1)
    );
    for (id, expected) in held {
        assert_eq!(context.get(id), Ok(expected.as_bytes()));
    }
    assert_eq!(context.get(discarded), Err(HeapErrorKind::StaleReference));
    assert_eq!(
        context
            .object_heap_usage()
            .expect("Objects")
            .live_array_values,
        1
    );
    drop(context);
    assert_eq!(
        state.objects.field_count(garbage),
        Err(ObjectHeapErrorKind::StaleReference)
    );
    assert_eq!(state.objects.field_count(child), Ok(1));
    assert_eq!(state.objects.field_count(game_child), Ok(1));
}

fn attach_payloads(
    state: &mut script_root_support::State,
    roots: &[ObjectId],
) -> Vec<(StringId, String)> {
    let work = HeapWorkBudget::default();
    let mut held = Vec::new();
    for (index, id) in roots.iter().copied().enumerate() {
        let text = format!("private root payload {index}");
        let string = state
            .strings
            .allocate(text.as_bytes(), work)
            .expect("Held string")
            .id;
        state
            .objects
            .put(
                id,
                b"payload",
                KernelValue::String(string),
                &state.strings,
                work,
            )
            .expect("Root graph string");
        held.push((string, text));
    }
    held
}
