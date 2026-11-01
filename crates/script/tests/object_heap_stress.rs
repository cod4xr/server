// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests exercise wide and repeatedly reused object storage."
)]
use crate::object_heap_support::{allocate, number, stores};
use cod4x_script::heap::{HeapWorkBudget, ObjectHeapBudget};

#[test]
fn object_store_exceeds_native_parent_and_child_pool_widths_without_truncation() {
    let (strings, mut objects) = stores(ObjectHeapBudget::default());
    let mut roots = Vec::new();
    for _ in 0..70_000 {
        roots.push(allocate(&mut objects));
    }
    let root = roots[0];
    for key in 0_i32..70_000 {
        objects
            .put(
                root,
                &key.to_le_bytes(),
                number(key),
                &strings,
                HeapWorkBudget::default(),
            )
            .expect("Wide fields");
    }
    assert_eq!(objects.usage().live_objects, 70_000);
    assert_eq!(objects.field_count(root), Ok(70_000));
    assert_eq!(
        objects
            .reclaim(&roots, HeapWorkBudget::default())
            .expect("Wide roots")
            .objects,
        0
    );
    for key in [0_i32, 32_768, 65_535, 69_999] {
        let equal_key = key.to_le_bytes().to_vec();
        assert_eq!(
            objects
                .get(root, &equal_key, &strings, HeapWorkBudget::default())
                .expect("Content lookup")
                .value,
            Some(number(key))
        );
        assert_eq!(
            objects
                .put(
                    root,
                    &equal_key,
                    number(-key),
                    &strings,
                    HeapWorkBudget::default()
                )
                .expect("Replace")
                .value,
            Some(number(key))
        );
        assert_eq!(
            objects
                .remove(root, &equal_key, &strings, HeapWorkBudget::default())
                .expect("Remove")
                .value,
            Some(number(-key))
        );
    }
    assert_eq!(objects.field_count(root), Ok(69_996));
    let collection = objects
        .reclaim(&[], HeapWorkBudget::default())
        .expect("Wide sweep");
    assert_eq!((collection.objects, collection.fields), (70_000, 69_996));
    let slots = objects.usage().object_slots;
    allocate(&mut objects);
    assert_eq!(objects.usage().object_slots, slots);
}

#[test]
fn object_churn_reuses_slots_and_rebuilds_tombstones_without_retaining_extra_buckets() {
    let budget = ObjectHeapBudget {
        max_live_objects: 2,
        max_object_slots: 2,
        max_live_fields: 2,
        max_field_slots: 2,
        max_index_buckets: 8,
        max_key_bytes: 16,
        ..ObjectHeapBudget::default()
    };
    let (strings, mut objects) = stores(budget);
    let root = allocate(&mut objects);
    objects
        .put(
            root,
            b"anchor",
            number(1),
            &strings,
            HeapWorkBudget::default(),
        )
        .expect("Anchor");
    for key in 0_u64..100_000 {
        let temporary = allocate(&mut objects);
        objects
            .put(
                root,
                &key.to_le_bytes(),
                number(2),
                &strings,
                HeapWorkBudget::default(),
            )
            .expect("Churn insert");
        assert_eq!(
            objects
                .remove(
                    root,
                    &key.to_le_bytes(),
                    &strings,
                    HeapWorkBudget::default()
                )
                .expect("Churn remove")
                .value,
            Some(number(2))
        );
        assert_eq!(
            objects
                .reclaim(&[root], HeapWorkBudget::default())
                .expect("Churn reclaim")
                .objects,
            1
        );
        assert!(objects.field_count(temporary).is_err());
    }
    let usage = objects.usage();
    assert_eq!(
        (usage.object_slots, usage.field_slots, usage.index_buckets),
        (2, 2, 8)
    );
    assert_eq!(
        (usage.live_objects, usage.live_fields, usage.key_bytes),
        (1, 1, 6)
    );
    assert_eq!(
        objects
            .get(root, b"anchor", &strings, HeapWorkBudget::default())
            .expect("Anchor survives")
            .value,
        Some(number(1))
    );
}

#[test]
fn object_large_keys_are_owned_and_admitted_by_content_work_without_text_conversion() {
    let (strings, mut objects) = stores(ObjectHeapBudget::default());
    let root = allocate(&mut objects);
    let mut key = vec![0xff; 1024 * 1024];
    key[0] = 0;
    objects
        .put(root, &key, number(42), &strings, HeapWorkBudget::default())
        .expect("Large binary key");
    let equal = key.clone();
    key.fill(0);
    let result = objects
        .get(root, &equal, &strings, HeapWorkBudget::default())
        .expect("Complete key equality");
    assert_eq!(result.value, Some(number(42)));
    assert!(result.usage.steps >= (equal.len() * 2) as u64);
    assert_eq!(
        objects
            .get(root, &key, &strings, HeapWorkBudget::default())
            .expect("Changed key is distinct")
            .value,
        None
    );
    assert!(
        objects
            .get(
                root,
                &equal,
                &strings,
                HeapWorkBudget {
                    max_steps: result.usage.steps - 1,
                    ..HeapWorkBudget::default()
                }
            )
            .is_err()
    );
    assert_eq!(objects.field_count(root), Ok(1));
}
