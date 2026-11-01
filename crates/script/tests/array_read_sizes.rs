// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests replay all native array size rows against real populated arrays rather than mocked counts."
)]
use crate::kernel_call_support::{definition, integer, verify};
use crate::object_heap_support::{number, stores};
use cod4x_script::execution::{
    KernelCallBudget, KernelHeapContext, KernelInstruction, KernelValue,
};
use cod4x_script::heap::{ArrayKey, HeapWorkBudget, ObjectHeapBudget};

#[test]
fn native_array_size_rows_replay_real_empty_and_populated_counts_including_the_u16_edge()
-> Result<(), String> {
    let module = verify(&[
        definition(
            &[
                KernelInstruction::LoadLocal(0),
                KernelInstruction::Size,
                KernelInstruction::Return,
            ],
            1,
            1,
        ),
        definition(&[integer(1), KernelInstruction::Return], 0, 0),
    ]);
    let work = HeapWorkBudget::default();
    let (mut replayed, mut other) = (0, 0);
    for line in include_str!("fixtures/object_size_vectors.txt")
        .lines()
        .filter(|line| !line.starts_with('#'))
    {
        let fields: Vec<_> = line.split_whitespace().collect();
        assert_eq!(fields.len(), 4);
        assert_eq!(fields[0], "O");
        let parent = u32::from_str_radix(fields[1], 16).map_err(|error| error.to_string())?;
        let size = fields[2]
            .parse::<u16>()
            .map_err(|error| error.to_string())?;
        let expected = fields[3]
            .parse::<i32>()
            .map_err(|error| error.to_string())?;
        if parent & 31 != 21 {
            assert_eq!(expected, 1);
            other += 1;
            continue;
        }
        assert_eq!(expected, i32::from(size));
        let (mut strings, mut objects) = stores(ObjectHeapBudget::default());
        let host = objects.new_array_value(work).expect("Host array").value;
        let data = objects.array_data(host, work).expect("Data").value;
        for index in 0..u64::from(size) {
            objects
                .array_put_unique(
                    host,
                    ArrayKey::Index(index),
                    KernelValue::Undefined,
                    &strings,
                    work,
                )
                .expect("Real raw undefined field");
        }
        let mut context =
            KernelHeapContext::new(&mut strings, &[], work).with_objects(&mut objects, &[]);
        let result = module
            .execute_with_heap(
                0,
                &[KernelValue::Array(host)],
                KernelCallBudget::default(),
                &mut context,
            )
            .expect("Size of populated host array");
        assert_eq!(result.value, number(expected));
        drop(context);
        assert_eq!(
            objects
                .array_owner_count(data, work)
                .expect("Host owner")
                .value,
            1
        );
        assert_eq!(
            objects.array_len(data, work).expect("Unchanged").value,
            u64::from(size)
        );
        replayed += 1;
    }
    assert_eq!((replayed, other), (15, 165));
    Ok(())
}
