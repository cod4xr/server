// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Helpers normalize explicit synthetic native key/string/object identities for checked read replay."
)]
use cod4x_script::execution::{KernelHeapContext, KernelScriptRoots, KernelValue};
use cod4x_script::heap::{
    ArrayId, ArrayKey, ArrayValueId, HeapWorkBudget, ObjectHeap, ObjectHeapBudget, ObjectId,
    StringHeap, StringHeapBudget, StringId,
};
use cod4x_script::legacy::LegacyNumber;

pub struct State {
    pub strings: StringHeap,
    pub objects: ObjectHeap,
    pub array: ArrayValueId,
    pub data: ArrayId,
    pub shared: ObjectId,
    pub plain: StringId,
    pub localized: StringId,
}

pub fn signed(word: u32) -> i32 {
    i32::from_ne_bytes(word.to_ne_bytes())
}
pub fn number(value: i32) -> KernelValue {
    KernelValue::Number(LegacyNumber::Integer(value))
}

pub fn byte_key(label: u32) -> Option<&'static [u8]> {
    Some(match label {
        11 => b"",
        12 => b"Case",
        13 => b"case",
        14 => b"missing",
        15 => b"last",
        65535 => b"UpperEdge",
        _ => return None,
    })
}

pub fn state(rotation: usize, aliases: u64) -> State {
    let mut strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let work = HeapWorkBudget::default();
    let plain = strings
        .allocate(b"Payload101", work)
        .expect("Plain payload")
        .id;
    let localized = strings
        .allocate(b"Payload102", work)
        .expect("Localized payload")
        .id;
    let mut objects = ObjectHeap::new(&strings, ObjectHeapBudget::default()).expect("Objects");
    let array = objects.new_array_value(work).expect("Host array").value;
    let data = objects.array_data(array, work).expect("Data").value;
    let shared = objects.allocate(work).expect("Shared ordinary object").id;
    objects
        .new_array_value(work)
        .expect("Separate modeled spare array");
    let entries = [
        (ArrayKey::Index(0), number(7)),
        (
            ArrayKey::Index(2),
            KernelValue::Number(LegacyNumber::Float(f32::from_bits(0x8000_0000))),
        ),
        (ArrayKey::Index(65535), KernelValue::Undefined),
        (ArrayKey::Index(0x7f_ffff), KernelValue::String(plain)),
        (
            ArrayKey::Bytes(byte_key(11).expect("Known key")),
            KernelValue::LocalizedString(localized),
        ),
        (
            ArrayKey::Bytes(byte_key(12).expect("Known key")),
            KernelValue::Object(shared),
        ),
        (
            ArrayKey::Bytes(byte_key(13).expect("Known key")),
            number(13),
        ),
        (
            ArrayKey::Bytes(byte_key(65535).expect("Known key")),
            number(-123),
        ),
        (
            ArrayKey::Bytes(byte_key(15).expect("Known key")),
            number(17),
        ),
    ];
    for index in 0..entries.len() {
        let (key, value) = entries[(index + rotation) % entries.len()];
        objects
            .array_put_unique(array, key, value, &strings, work)
            .expect("Current profile entry");
    }
    for _ in 1..aliases {
        objects
            .copy_array_value(array, work)
            .expect("Retained host alias");
    }
    State {
        strings,
        objects,
        array,
        data,
        shared,
        plain,
        localized,
    }
}

impl State {
    pub fn context(&mut self, work: HeapWorkBudget) -> KernelHeapContext<'_> {
        let roots = KernelScriptRoots::new(self.shared, self.shared, self.array, self.shared);
        KernelHeapContext::new(&mut self.strings, &[], work)
            .with_objects(&mut self.objects, &[])
            .with_script_roots(roots)
    }

    pub fn key_value(&mut self, tag: u32, word: u32) -> Result<KernelValue, String> {
        Ok(match tag {
            6 => number(signed(word)),
            2 => KernelValue::String(
                self.strings
                    .allocate(
                        byte_key(word).ok_or("Unknown normalized key")?,
                        HeapWorkBudget::default(),
                    )
                    .expect("Key bytes")
                    .id,
            ),
            _ => return Err("Unknown native read key tag".into()),
        })
    }

    pub fn keys(&self) -> Vec<String> {
        self.objects
            .array_keys(self.data, HeapWorkBudget::default())
            .expect("Keys")
            .value
            .iter()
            .map(|key| match key {
                ArrayKey::Index(value) => format!("i:{value}"),
                ArrayKey::Bytes(bytes) => {
                    let label = [11, 12, 13, 14, 15, 65535]
                        .into_iter()
                        .find(|label| byte_key(*label) == Some(*bytes))
                        .expect("Normalized key");
                    format!("s:{label}")
                }
            })
            .collect()
    }

    pub fn assert_host_owners(&self, aliases: u64) {
        assert_eq!(
            self.objects
                .array_owner_count(self.data, HeapWorkBudget::default())
                .expect("Owners")
                .value,
            aliases
        );
        assert_eq!(self.objects.usage().live_array_values, aliases + 1);
        assert_eq!(self.objects.usage().live_objects, 3);
        assert_eq!(self.objects.usage().live_fields, 9);
    }
}
