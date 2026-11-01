// SPDX-License-Identifier: AGPL-3.0-or-later
//! Native primitive delete outcomes through typed cells; no native VM/freeing/interner parity.
#![allow(
    clippy::expect_used,
    reason = "Tests replay all primitive D outcomes with explicit key identities and transient field-slot normalization."
)]
use crate::kernel_call_support::{self, definition, integer};
use crate::object_heap_support::{number, stores};
use cod4x_script::execution::{
    KernelCallBudget, KernelHeapContext, KernelInstruction, KernelValue,
};
use cod4x_script::heap::{ArrayKey, HeapWorkBudget, ObjectHeapBudget};

fn decimal(value: &str) -> u64 {
    value.parse().expect("Native decimal")
}

fn flag(value: &str) -> bool {
    let value = decimal(value);
    assert!(value <= 1, "Native boolean profile");
    value == 1
}

fn normalized_keys(keys: &[ArrayKey<'_>]) -> Vec<String> {
    keys.iter()
        .map(|key| match key {
            ArrayKey::Index(index) => format!("i:{index}"),
            ArrayKey::Bytes(bytes) => format!(
                "s:{}",
                std::str::from_utf8(bytes)
                    .expect("Synthetic ASCII keys")
                    .strip_prefix("key")
                    .expect("Explicit key label")
            ),
        })
        .collect()
}

fn code(shared: bool, dynamic: bool, original: bool) -> Vec<KernelInstruction> {
    let mut code = vec![
        KernelInstruction::EmptyArray,
        KernelInstruction::StoreLocal(4),
    ];
    for index in 0..3 {
        code.extend([
            integer(i32::try_from(index).expect("Small fixture index")),
            KernelInstruction::LoadLocal(index),
            KernelInstruction::StoreArrayLocal(4),
        ]);
    }
    if shared {
        code.extend([
            KernelInstruction::LoadLocal(4),
            KernelInstruction::StoreLocal(5),
        ]);
    }
    if dynamic {
        code.push(KernelInstruction::Constant(KernelValue::Undefined));
    }
    code.push(KernelInstruction::LoadLocal(3));
    code.push(if dynamic {
        KernelInstruction::StoreArrayLocal(4)
    } else {
        KernelInstruction::ClearArrayLocal(4)
    });
    code.extend([
        KernelInstruction::LoadLocal(if original { 5 } else { 4 }),
        KernelInstruction::Return,
    ]);
    code
}

fn replay(row: &[&str], original: bool) -> Result<(), String> {
    let shared = flag(row[1]);
    let strings_as_keys = flag(row[2]);
    let key = decimal(row[3]);
    assert!(matches!(key, 0 | 1 | 2 | 7));
    let dynamic = flag(row[4]);
    let peak = decimal(row[5]);
    let added = decimal(row[7]);
    assert_eq!(peak, 3 + u64::from(key == 7 && dynamic));
    assert_eq!(
        added,
        3 * u64::from(shared) + u64::from(key == 7 && dynamic)
    );
    assert_eq!(
        decimal(row[6]),
        if strings_as_keys {
            if dynamic || key != 7 { 2 } else { 1 }
        } else {
            0
        }
    );
    let (mut strings, mut objects) = stores(ObjectHeapBudget {
        max_live_fields: 3 * u64::from(shared) + peak,
        ..ObjectHeapBudget::default()
    });
    let work = HeapWorkBudget::default();
    let mut arguments = Vec::new();
    for key in [0, 1, 2, key] {
        arguments.push(if strings_as_keys {
            KernelValue::String(
                strings
                    .allocate(format!("key{}", key + 100).as_bytes(), work)
                    .expect("Normalized native key")
                    .id,
            )
        } else {
            number(i32::try_from(key).expect("Fixture key"))
        });
    }
    let code = code(shared, dynamic, original);
    let module = kernel_call_support::verify(&[definition(&code, 4, 6)]);
    let mut context =
        KernelHeapContext::new(&mut strings, &[], work).with_objects(&mut objects, &[]);
    let result = module
        .execute_with_heap(0, &arguments, KernelCallBudget::default(), &mut context)
        .expect("Native primitive deletion");
    let KernelValue::Array(value) = result.value else {
        return Err("Expected returned owner".into());
    };
    let usage = context.object_heap_usage().expect("Objects");
    assert_eq!(usage.live_array_values, 1);
    assert_eq!(
        usage.field_slots,
        3 + added,
        "Transient computed child and COW copies are real slots"
    );
    drop(context);
    let data = objects
        .array_data(value, work)
        .expect("Returned data")
        .value;
    let keys = objects.array_keys(data, work).expect("Sibling order");
    let normalized = normalized_keys(&keys.value);
    let expected: Vec<_> = if original {
        [2, 1, 0]
            .into_iter()
            .map(|key| {
                if strings_as_keys {
                    format!("s:{}", key + 100)
                } else {
                    format!("i:{key}")
                }
            })
            .collect()
    } else {
        row[8..].iter().map(|key| (*key).to_owned()).collect()
    };
    assert_eq!(normalized, expected);
    assert_eq!(
        objects.array_len(data, work).expect("Count").value,
        u64::try_from(expected.len()).expect("Count")
    );
    assert_eq!(
        objects
            .array_owner_count(data, work)
            .expect("Transferred owner")
            .value,
        1
    );
    objects
        .release_array_value(value, work)
        .expect("Host release");
    assert_eq!(
        objects
            .reclaim(&[], work)
            .expect("No retained VM aliases")
            .objects,
        1 + u64::from(shared)
    );
    Ok(())
}

#[test]
fn all_native_primitive_delete_rows_replay_through_unique_and_copied_typed_local_cells()
-> Result<(), String> {
    let mut count = 0;
    for line in include_str!("fixtures/array_vectors.txt")
        .lines()
        .filter(|line| line.starts_with("D "))
    {
        let row: Vec<_> = line.split_whitespace().collect();
        assert!((10..=11).contains(&row.len()));
        replay(&row, false)?;
        if flag(row[1]) {
            replay(&row, true)?;
        }
        count += 1;
    }
    assert_eq!(count, 32);
    Ok(())
}
