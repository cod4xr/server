// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Private tests force probe collisions and verify exact failure work without public identity exposure."
)]
use super::OwnerTable;
use crate::execution::{KernelErrorKind, KernelResource};
use crate::heap::{HeapWorkBudget, ObjectHeap, ObjectHeapBudget, StringHeap, StringHeapBudget};

#[test]
fn collision_chains_keep_identity_across_deletion_and_every_short_probe_is_read_only() {
    let strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let mut objects = ObjectHeap::new(&strings, ObjectHeapBudget::default()).expect("Objects");
    let values: Vec<_> = (0..7)
        .map(|_| {
            objects
                .new_array_value(HeapWorkBudget::default())
                .expect("Host")
                .value
        })
        .collect();
    let mut table = OwnerTable::allocate(8).expect("Eight explicit buckets");
    for value in values.iter().take(6).copied() {
        let found = table
            .lookup_from(value, 0, |_| Ok(()))
            .expect("Forced chain");
        assert!(!found.present);
        table.insert(found.slot, value);
    }
    let found = table.lookup_from(values[0], 0, |_| Ok(())).expect("First");
    table.remove(found.slot);
    let found = table.lookup_from(values[3], 0, |_| Ok(())).expect("Middle");
    table.remove(found.slot);
    let before = table.values.clone();
    let flags = table.used.clone();
    for maximum in 0..6 {
        let mut charged = 0;
        table
            .lookup_from(values[5], 0, |count| {
                let next = charged + count;
                if next > maximum {
                    return Err(KernelErrorKind::BudgetExceeded {
                        resource: KernelResource::HeapWorkSteps,
                        requested: next,
                        maximum,
                    });
                }
                charged = next;
                Ok(())
            })
            .expect_err("Every shorter probe allowance");
        assert_eq!(charged, maximum);
        assert_eq!(table.values, before);
        assert_eq!(table.used, flags);
    }
    let mut charged = 0;
    assert!(
        table
            .lookup_from(values[5], 0, |count| {
                charged += count;
                Ok(())
            })
            .expect("Complete chain")
            .present
    );
    assert_eq!(charged, 6);
    let found = table
        .lookup_from(values[6], 0, |_| Ok(()))
        .expect("Absent identity");
    assert!(!found.present);
    assert_eq!(found.slot, 0);
    table.insert(found.slot, values[6]);
    assert_eq!((table.len, table.filled), (5, 6));
    assert_eq!(table.replacement(1), Ok(Some(8)));
    assert_eq!(table.replacement(2), Ok(Some(16)));
    assert!(
        table
            .lookup_from(values[5], 0, |_| Ok(()))
            .expect("Later identity")
            .present
    );
    assert!(
        !table
            .lookup_from(values[3], 0, |_| Ok(()))
            .expect("Removed identity")
            .present
    );
}

#[test]
fn complete_tombstone_ring_terminates_and_reuses_a_bucket() {
    let strings = StringHeap::new(StringHeapBudget::default()).expect("Strings");
    let mut objects = ObjectHeap::new(&strings, ObjectHeapBudget::default()).expect("Objects");
    let value = objects
        .new_array_value(HeapWorkBudget::default())
        .expect("Host")
        .value;
    let mut table = OwnerTable::allocate(8).expect("Table");
    table.used.fill(true);
    table.filled = 8;
    let mut work = 0;
    let found = table
        .lookup_from(value, 7, |count| {
            work += count;
            Ok(())
        })
        .expect("Finite complete ring");
    assert_eq!(work, 8);
    assert!(!found.present);
    assert_eq!(found.slot, 7);
    table.insert(found.slot, value);
    assert_eq!((table.len, table.filled), (1, 8));
    assert_eq!(table.replacement(1), Ok(Some(8)));
}
