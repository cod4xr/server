// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Independent hostile-operation model checks every live field."
)]
use std::collections::{BTreeMap, HashMap, HashSet};

use cod4x_script::execution::KernelValue;
use cod4x_script::heap::{HeapWorkBudget, ObjectHeap, ObjectId, StringHeap};

#[derive(Default)]
pub struct Model {
    pub live: Vec<ObjectId>,
    pub stale: Vec<ObjectId>,
    pub fields: HashMap<ObjectId, BTreeMap<Vec<u8>, KernelValue>>,
}

impl Model {
    pub fn add(&mut self, id: ObjectId) {
        assert!(!self.fields.contains_key(&id));
        self.live.push(id);
        self.fields.insert(id, BTreeMap::new());
    }

    pub fn collect(&mut self, roots: &[ObjectId]) -> (u64, u64) {
        let before_objects = self.live.len();
        let before_fields: usize = self.fields.values().map(BTreeMap::len).sum();
        let mut marked = HashSet::new();
        let mut queue = roots.to_vec();
        while let Some(id) = queue.pop() {
            if marked.insert(id) {
                for value in self.fields.get(&id).expect("Valid model graph").values() {
                    if let KernelValue::Object(child) = value {
                        queue.push(*child);
                    }
                }
            }
        }
        self.live.retain(|id| {
            if marked.contains(id) {
                true
            } else {
                self.stale.push(*id);
                false
            }
        });
        self.fields.retain(|id, _| marked.contains(id));
        let after_fields: usize = self.fields.values().map(BTreeMap::len).sum();
        (
            (before_objects - self.live.len()) as u64,
            (before_fields - after_fields) as u64,
        )
    }

    pub fn check(&self, objects: &ObjectHeap, strings: &StringHeap) {
        let fields: u64 = self.fields.values().map(|fields| fields.len() as u64).sum();
        let key_bytes: u64 = self
            .fields
            .values()
            .flat_map(BTreeMap::keys)
            .map(|key| key.len() as u64)
            .sum();
        assert_eq!(objects.usage().live_objects, self.live.len() as u64);
        assert_eq!(objects.usage().live_fields, fields);
        assert_eq!(objects.usage().key_bytes, key_bytes);
        for (id, fields) in &self.fields {
            assert_eq!(objects.field_count(*id), Ok(fields.len() as u64));
            for (key, value) in fields {
                assert_eq!(
                    objects
                        .get(*id, key, strings, HeapWorkBudget::default())
                        .expect("Model field")
                        .value,
                    Some(*value)
                );
            }
            assert_eq!(
                objects
                    .get(*id, b"absent", strings, HeapWorkBudget::default())
                    .expect("Absent model field")
                    .value,
                None
            );
        }
    }
}
