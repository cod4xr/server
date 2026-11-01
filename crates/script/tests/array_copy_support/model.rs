// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Independent version/order model accepts admitted synthetic ownership operations."
)]
use std::collections::BTreeMap;

use crate::array_copy_support::model_step;
use crate::object_heap_support::stores;
use cod4x_script::execution::KernelValue;
use cod4x_script::heap::{
    ArrayId, ArrayKey, ArrayValueId, HeapWorkBudget, ObjectHeap, ObjectHeapBudget, StringHeap,
};

pub(super) type Key = (u64, u64);

pub(super) struct Version {
    pub(super) data: ArrayId,
    pub(super) entries: BTreeMap<Key, KernelValue>,
    pub(super) order: Vec<Key>,
    pub(super) alive: bool,
}

pub(super) struct Model {
    pub(super) versions: Vec<Version>,
    pub(super) values: Vec<(ArrayValueId, usize)>,
}

pub fn run() {
    let (strings, mut heap) = stores(ObjectHeapBudget::default());
    let mut model = Model {
        versions: Vec::new(),
        values: Vec::new(),
    };
    let mut random = 0x18a4_2f01_bc6d_93e7_u64;
    for step in 0..16_384 {
        random = random
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        model_step::step(&mut model, &mut heap, &strings, random);
        if step % 31 == 0 {
            collect(&mut model, &mut heap);
        }
        check(&model, &heap, &strings);
    }
    for (value, _) in model.values.drain(..) {
        heap.release_array_value(value, HeapWorkBudget::default())
            .expect("Final release");
    }
    collect(&mut model, &mut heap);
    assert_eq!(
        (
            heap.usage().live_arrays,
            heap.usage().live_fields,
            heap.usage().live_array_values
        ),
        (0, 0, 0)
    );
}

fn collect(model: &mut Model, heap: &mut ObjectHeap) {
    let (mut dead, mut fields) = (Vec::new(), 0);
    for (label, version) in model.versions.iter_mut().enumerate() {
        if version.alive && !model.values.iter().any(|(_, current)| *current == label) {
            dead.push(version.data);
            fields += version.entries.len() as u64;
            version.alive = false;
        }
    }
    let budget = HeapWorkBudget::default();
    let collection = heap.reclaim(&[], budget).expect("Model garbage collection");
    assert_eq!(
        (collection.objects, collection.fields),
        (dead.len() as u64, fields)
    );
    for data in dead {
        assert!(heap.retain_array(data, budget).is_err());
    }
}

fn check(model: &Model, heap: &ObjectHeap, strings: &StringHeap) {
    let budget = HeapWorkBudget::default();
    let (mut arrays, mut fields) = (0, 0);
    for (label, version) in model
        .versions
        .iter()
        .enumerate()
        .filter(|(_, version)| version.alive)
    {
        arrays += 1;
        fields += version.entries.len() as u64;
        let owners = model
            .values
            .iter()
            .filter(|(_, current)| *current == label)
            .count() as u64;
        assert_eq!(
            heap.array_owner_count(version.data, budget)
                .expect("Modeled owners")
                .value,
            owners
        );
        assert_eq!(
            heap.array_len(version.data, budget)
                .expect("Modeled length")
                .value,
            version.entries.len() as u64
        );
        let actual: Vec<_> = heap
            .array_keys(version.data, budget)
            .expect("Modeled order")
            .value
            .into_iter()
            .map(|key| match key {
                ArrayKey::Index(index) => (0, index),
                ArrayKey::Bytes(bytes) => (
                    1,
                    u64::from_le_bytes(bytes.try_into().expect("Modeled binary key")),
                ),
            })
            .collect();
        assert_eq!(actual, version.order);
        for ((kind, index), expected) in &version.entries {
            let bytes = index.to_le_bytes();
            let key = if *kind == 0 {
                ArrayKey::Index(*index)
            } else {
                ArrayKey::Bytes(&bytes)
            };
            assert_eq!(
                heap.array_get(version.data, key, strings, budget)
                    .expect("Complete model payload")
                    .value,
                Some(*expected)
            );
        }
    }
    for (value, label) in &model.values {
        assert_eq!(
            heap.array_data(*value, budget).expect("Modeled data").value,
            model.versions[*label].data
        );
    }
    let usage = heap.usage();
    assert_eq!(
        (
            usage.live_arrays,
            usage.live_fields,
            usage.live_array_values
        ),
        (arrays, fields, model.values.len() as u64)
    );
}
