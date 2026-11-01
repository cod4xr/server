// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests compare mixed checked operations with an independent graph model."
)]
use crate::object_heap_support::model::Model;
use crate::object_heap_support::{allocate, number, stores};
use cod4x_script::execution::KernelValue;
use cod4x_script::heap::{
    HeapWorkBudget, ObjectHeap, ObjectHeapBudget, ObjectHeapError, ObjectHeapErrorKind, ObjectId,
    StringHeap,
};

fn next(state: &mut u64) -> usize {
    *state = state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    usize::try_from(*state >> 32).expect("32-bit selection")
}

fn random_budget(state: &mut u64) -> HeapWorkBudget {
    match next(state) % 4 {
        0 => HeapWorkBudget {
            max_steps: 0,
            ..HeapWorkBudget::default()
        },
        1 => HeapWorkBudget {
            max_steps: 24,
            max_scratch_bytes: 0,
        },
        _ => HeapWorkBudget::default(),
    }
}

fn rejected(error: ObjectHeapError, budget: HeapWorkBudget) {
    assert!(matches!(
        error.kind,
        ObjectHeapErrorKind::BudgetExceeded { .. }
            | ObjectHeapErrorKind::ForeignReference
            | ObjectHeapErrorKind::StaleReference
    ));
    assert!(error.usage.steps <= budget.max_steps);
    assert!(error.usage.scratch_bytes <= budget.max_scratch_bytes);
}

fn mutate(
    objects: &mut ObjectHeap,
    strings: &StringHeap,
    model: &mut Model,
    id: ObjectId,
    value: KernelValue,
    selection: usize,
    budget: HeapWorkBudget,
) {
    let key = ((selection >> 8) % 32).to_le_bytes();
    let before = objects.usage();
    let valid = model.fields.contains_key(&id)
        && (!matches!(value, KernelValue::Object(child) if !model.fields.contains_key(&child)));
    let result = if selection & 1 == 0 {
        objects.put(id, &key, value, strings, budget)
    } else {
        objects.remove(id, &key, strings, budget)
    };
    match result {
        Ok(result) => {
            assert!(model.fields.contains_key(&id));
            let fields = model.fields.get_mut(&id).expect("Valid parent");
            let expected = if selection & 1 == 0 {
                assert!(valid);
                fields.insert(key.to_vec(), value)
            } else {
                fields.remove(key.as_slice())
            };
            assert_eq!(result.value, expected);
        }
        Err(error) => {
            rejected(error, budget);
            assert_eq!(objects.usage(), before);
        }
    }
}

fn collect(
    objects: &mut ObjectHeap,
    model: &mut Model,
    roots: &[ObjectId],
    budget: HeapWorkBudget,
) {
    let before = objects.usage();
    match objects.reclaim(roots, budget) {
        Ok(result) => {
            assert!(roots.iter().all(|id| model.fields.contains_key(id)));
            assert_eq!((result.objects, result.fields), model.collect(roots));
        }
        Err(error) => {
            rejected(error, budget);
            assert_eq!(objects.usage(), before);
        }
    }
}

#[test]
fn object_heap_matches_independent_graph_model_through_4096_hostile_mixed_operations() {
    let (mut strings, mut objects) = stores(ObjectHeapBudget {
        max_live_objects: 128,
        max_object_slots: 128,
        max_live_fields: 512,
        max_field_slots: 512,
        max_key_bytes: 4096,
        ..ObjectHeapBudget::default()
    });
    let plain = strings
        .allocate(b"plain", HeapWorkBudget::default())
        .expect("Plain")
        .id;
    let localized = strings
        .allocate(b"MENU", HeapWorkBudget::default())
        .expect("Localized")
        .id;
    let (_, mut foreign) = stores(ObjectHeapBudget::default());
    let foreign_id = allocate(&mut foreign);
    let mut model = Model::default();
    let mut random = 0x0192_c84d_ef63_8ab1;
    let mut peak_objects = 0;
    let mut peak_fields = 0;
    let mut saw_object = false;
    let mut saw_localized = false;
    for iteration in 0..4096 {
        let selection = next(&mut random);
        let budget = random_budget(&mut random);
        if model.live.is_empty() || selection.is_multiple_of(8) {
            let before = objects.usage();
            match objects.allocate(budget) {
                Ok(result) => model.add(result.id),
                Err(error) => {
                    rejected(error, budget);
                    assert_eq!(objects.usage(), before);
                }
            }
        } else {
            let live = model.live[next(&mut random) % model.live.len()];
            let id = if iteration % 17 == 0 {
                foreign_id
            } else if iteration % 19 == 0 && !model.stale.is_empty() {
                model.stale[next(&mut random) % model.stale.len()]
            } else {
                live
            };
            if selection % 8 == 1 {
                collect(&mut objects, &mut model, &[live, id, live], budget);
            } else {
                let child = model.live[next(&mut random) % model.live.len()];
                let value = match next(&mut random) % 6 {
                    0 => KernelValue::Undefined,
                    1 => KernelValue::Object(child),
                    2 => KernelValue::String(plain),
                    3 => KernelValue::LocalizedString(localized),
                    4 => KernelValue::Object(foreign_id),
                    _ => number(iteration),
                };
                mutate(
                    &mut objects,
                    &strings,
                    &mut model,
                    id,
                    value,
                    selection,
                    budget,
                );
            }
        }
        if iteration % 31 == 0 {
            objects
                .reclaim_strings(&mut strings, &[plain, localized], HeapWorkBudget::default())
                .expect("All stored and external strings rooted");
        }
        model.check(&objects, &strings);
        peak_objects = peak_objects.max(objects.usage().live_objects);
        peak_fields = peak_fields.max(objects.usage().live_fields);
        for value in model
            .fields
            .values()
            .flat_map(std::collections::BTreeMap::values)
        {
            saw_object |= matches!(value, KernelValue::Object(_));
            saw_localized |= matches!(value, KernelValue::LocalizedString(_));
        }
    }
    assert!(peak_objects >= 8 && peak_fields >= 8);
    assert!(saw_object && saw_localized && model.stale.len() >= 64);
    collect(&mut objects, &mut model, &[], HeapWorkBudget::default());
    model.check(&objects, &strings);
    assert_eq!(objects.usage().index_buckets, 0);
    assert_eq!(
        objects
            .reclaim_strings(&mut strings, &[], HeapWorkBudget::default())
            .expect("Release all strings")
            .strings,
        2
    );
}
