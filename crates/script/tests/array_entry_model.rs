// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests compare typed indexed updates/order with an independent ordered-map model."
)]
use crate::object_heap_support::{number, stores};
use cod4x_script::heap::{ArrayKey, HeapWorkBudget, ObjectHeapBudget};
use std::collections::BTreeMap;

#[test]
fn sixteen_thousand_mixed_typed_key_updates_match_independent_map_values_and_sibling_order() {
    let (strings, mut heap) = stores(ObjectHeapBudget::default());
    let budget = HeapWorkBudget::default();
    let value = heap.new_array_value(budget).expect("Value").value;
    let data = heap.array_data(value, budget).expect("Data").value;
    let (mut model, mut order) = (BTreeMap::new(), Vec::new());
    let mut random = 0x8aa9_4b51_83c0_2f17_u64;
    for step in 0..16_384 {
        random = random
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let label = ((random >> 60) % 2, (1 << 48) + ((random >> 20) % 41));
        let bytes = label.1.to_le_bytes();
        let key = if label.0 == 0 {
            ArrayKey::Index(label.1)
        } else {
            ArrayKey::Bytes(&bytes)
        };
        if random.is_multiple_of(3) {
            let expected = model.remove(&label);
            assert_eq!(
                heap.array_remove_unique(value, key, &strings, budget)
                    .expect("Remove")
                    .value,
                expected
            );
            order.retain(|current| *current != label);
        } else {
            let next = number(step);
            let expected = model.insert(label, next);
            if expected.is_none() {
                order.insert(0, label);
            }
            assert_eq!(
                heap.array_put_unique(value, key, next, &strings, budget)
                    .expect("Insert/replace")
                    .value,
                expected
            );
        }
        assert_eq!(
            heap.array_len(data, budget).expect("Model length").value,
            model.len() as u64
        );
        assert_eq!(
            heap.array_get(data, key, &strings, budget)
                .expect("Selected lookup")
                .value,
            model.get(&label).copied()
        );
        let keys = heap.array_keys(data, budget).expect("Model order");
        let labels: Vec<_> = keys
            .value
            .into_iter()
            .map(|key| match key {
                ArrayKey::Index(index) => (0, index),
                ArrayKey::Bytes(bytes) => (
                    1,
                    u64::from_le_bytes(bytes.try_into().expect("Exact modeled key bytes")),
                ),
            })
            .collect();
        assert_eq!(labels, order);
        if step % 127 == 0 {
            assert_eq!(heap.reclaim(&[], budget).expect("Managed root").fields, 0);
        }
    }
    for ((kind, index), expected) in model {
        let bytes = index.to_le_bytes();
        let key = if kind == 0 {
            ArrayKey::Index(index)
        } else {
            ArrayKey::Bytes(&bytes)
        };
        assert_eq!(
            heap.array_get(data, key, &strings, budget)
                .expect("Final independent lookup")
                .value,
            Some(expected)
        );
    }
}
