// SPDX-License-Identifier: AGPL-3.0-or-later
//! Selected low-word native helper results; modeled pool/ref/removal services, not VM parity.
#![allow(
    clippy::expect_used,
    reason = "Fixture replay checks explicitly classified native helper value domains."
)]
use crate::kernel_call_support::definition;
use crate::object_heap_support::{allocate, stores};
use cod4x_script::execution::{
    KernelExecutionBudget, KernelHeapContext, KernelInstruction, KernelValue,
    KernelVerificationBudget, VerifiedKernelProgram,
};
use cod4x_script::heap::{HeapWorkBudget, ObjectHeap, ObjectHeapBudget, StringHeap};
use cod4x_script::legacy::LegacyNumber;

#[test]
fn native_field_get_and_clear_traces_replay_with_explicit_helper_only_function_words()
-> Result<(), String> {
    let names = [b"native key".to_vec()];
    let read = [
        KernelInstruction::LoadLocal(0),
        KernelInstruction::LoadField(0),
        KernelInstruction::Return,
    ];
    let clear = [
        KernelInstruction::Constant(KernelValue::Undefined),
        KernelInstruction::LoadLocal(0),
        KernelInstruction::StoreField(0),
        KernelInstruction::LoadLocal(0),
        KernelInstruction::LoadField(0),
        KernelInstruction::Return,
    ];
    let mut read_definition = definition(&read, 1, 1);
    read_definition.field_names = &names;
    let reader = VerifiedKernelProgram::verify_definition(
        read_definition,
        KernelVerificationBudget::default(),
    )
    .expect("Read proof");
    let mut clear_definition = definition(&clear, 1, 1);
    clear_definition.field_names = &names;
    let clearer = VerifiedKernelProgram::verify_definition(
        clear_definition,
        KernelVerificationBudget::default(),
    )
    .expect("Clear proof");
    let mut replayed = 0;
    let mut helper_only = 0;
    for row in include_str!("fixtures/field_vectors.txt")
        .lines()
        .filter(|row| !row.starts_with('#'))
    {
        let columns: Vec<_> = row.split_whitespace().collect();
        assert_eq!(columns.len(), 5);
        assert_eq!(columns[0], "F");
        let tag: u32 = columns[1]
            .parse()
            .map_err(|error: std::num::ParseIntError| error.to_string())?;
        let word = u32::from_str_radix(columns[2], 16).map_err(|error| error.to_string())?;
        assert_eq!(columns[1], columns[3]);
        assert_eq!(columns[2], columns[4]);
        if tag == 9 {
            helper_only += 1;
            continue;
        }
        let (mut strings, mut objects) = stores(ObjectHeapBudget::default());
        let root = allocate(&mut objects);
        let expected = mapped_value(tag, word, &mut strings, &mut objects)?;
        objects
            .put(
                root,
                &names[0],
                expected,
                &strings,
                HeapWorkBudget::default(),
            )
            .expect("Raw setter retains Undefined separately from source assignment");
        let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
            .with_objects(&mut objects, &[]);
        let result = reader
            .execute_with_heap(
                &[KernelValue::Object(root)],
                KernelExecutionBudget::default(),
                &mut context,
            )
            .expect("Native getter subset")
            .value;
        match (result, expected) {
            (
                KernelValue::Number(LegacyNumber::Float(actual)),
                KernelValue::Number(LegacyNumber::Float(expected)),
            ) => assert_eq!(actual.to_bits(), expected.to_bits()),
            (actual, expected) => assert_eq!(actual, expected),
        }
        assert_eq!(
            clearer
                .execute_with_heap(
                    &[KernelValue::Object(root)],
                    KernelExecutionBudget::default(),
                    &mut context
                )
                .expect("VM delete then missing")
                .value,
            KernelValue::Undefined
        );
        assert_eq!(objects.field_count(root), Ok(0));
        replayed += 1;
    }
    assert_eq!((replayed, helper_only), (27, 8));
    Ok(())
}

fn mapped_value(
    tag: u32,
    word: u32,
    strings: &mut StringHeap,
    objects: &mut ObjectHeap,
) -> Result<KernelValue, String> {
    Ok(match tag {
        0 => KernelValue::Undefined,
        1 => KernelValue::Object(allocate(objects)),
        2 | 3 => {
            let id = strings
                .allocate(b"modeled string", HeapWorkBudget::default())
                .expect("String mapping")
                .id;
            if tag == 2 {
                KernelValue::String(id)
            } else {
                KernelValue::LocalizedString(id)
            }
        }
        5 => KernelValue::Number(LegacyNumber::Float(f32::from_bits(word))),
        6 => KernelValue::Number(LegacyNumber::Integer(i32::from_ne_bytes(
            word.to_ne_bytes(),
        ))),
        _ => return Err("Unclassified native field domain".into()),
    })
}
