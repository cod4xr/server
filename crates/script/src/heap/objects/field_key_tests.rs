// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Private tests force colliding typed keys and malformed internal encodings."
)]
use crate::execution::KernelValue;
use crate::heap::objects::field_index::{Bucket, FieldIndex};
use crate::heap::objects::field_key::FieldKey;
use crate::heap::objects::object_slot::{Field, FieldSlot};
use crate::heap::objects::object_work::ObjectWork;
use crate::heap::{ArrayKey, HeapWorkBudget, ObjectHeapErrorKind};

#[test]
fn colliding_array_key_domains_compare_complete_encoding_without_allocating_lookup_keys()
-> Result<(), &'static str> {
    let keys = [
        ArrayKey::Index(0),
        ArrayKey::Bytes(&[0; 8]),
        ArrayKey::Bytes(b""),
        ArrayKey::Index(u64::MAX),
    ];
    let mut work = ObjectWork::new(HeapWorkBudget::default());
    let fields: Vec<_> = keys
        .iter()
        .map(|key| {
            FieldSlot::Occupied(Field {
                owner: 0,
                hash: 0,
                key: FieldKey::Array(*key)
                    .copy(&mut work)
                    .expect("Lossless encoding"),
                value: KernelValue::Undefined,
                previous: None,
                next: None,
            })
        })
        .collect();
    let index = FieldIndex {
        buckets: vec![
            Bucket::Occupied(0),
            Bucket::Occupied(1),
            Bucket::Occupied(2),
            Bucket::Occupied(3),
            Bucket::Empty,
            Bucket::Empty,
            Bucket::Empty,
            Bucket::Empty,
        ],
        live: 4,
        filled: 4,
    };
    for (expected, key) in keys.into_iter().enumerate() {
        assert_eq!(
            index
                .lookup_key(&fields, 0, 0, FieldKey::Array(key), &mut work)
                .expect("Forced same-hash comparison")
                .field,
            Some(expected)
        );
    }
    assert_eq!(
        index
            .lookup_key(
                &fields,
                0,
                0,
                FieldKey::Array(ArrayKey::Bytes(&[255; 8])),
                &mut work
            )
            .expect("Same length, different contents")
            .field,
        None
    );
    for malformed in [b"".as_slice(), &[0], &[0; 8], &[0; 10], &[2, 0]] {
        assert!(matches!(
            FieldKey::decode_array(malformed),
            Err(ObjectHeapErrorKind::InvalidState)
        ));
    }
    for (expected, field) in keys.into_iter().zip(fields) {
        let FieldSlot::Occupied(field) = field else {
            return Err("Expected occupied synthetic field");
        };
        assert!(FieldKey::Array(expected).matches(&field.key));
        assert!(
            FieldKey::Array(FieldKey::decode_array(&field.key).expect("Canonical roundtrip"))
                .matches(&field.key)
        );
    }
    Ok(())
}
