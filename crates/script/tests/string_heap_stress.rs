// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Finite deterministic wide/churn workloads admit their complete storage/work scopes."
)]

use cod4x_script::heap::{
    HeapErrorKind, HeapResource, HeapWorkBudget, StringHeap, StringHeapBudget,
};
use std::collections::HashSet;

#[test]
fn seventy_thousand_live_strings_preserve_distinct_wide_references_and_reclaim_exact_bytes() {
    let mut heap = StringHeap::new(StringHeapBudget::default()).expect("Wide store");
    let mut ids = Vec::with_capacity(70_000);
    for number in 0_u32..70_000 {
        ids.push(
            heap.allocate(&number.to_le_bytes(), HeapWorkBudget::default())
                .expect("Wide live value")
                .id,
        );
    }
    assert_eq!(ids.iter().copied().collect::<HashSet<_>>().len(), 70_000);
    for (number, id) in ids.iter().copied().enumerate() {
        assert_eq!(
            heap.get(id).expect("Exact wide payload"),
            u32::try_from(number).expect("Small value").to_le_bytes()
        );
    }
    let roots = [ids[0], ids[65_535], ids[69_999]];
    let collection = heap
        .reclaim(&roots, HeapWorkBudget::default())
        .expect("Wide complete roots");
    assert_eq!(collection.freed_strings, 69_997);
    assert_eq!(collection.freed_payload_bytes, 69_997 * 4);
    assert_eq!(collection.usage.steps, 210_003);
    assert_eq!(heap.usage().live_strings, 3);
    assert_eq!(heap.usage().payload_bytes, 12);
    assert_eq!(heap.usage().slots, 70_000);
    assert_eq!(heap.get(ids[65_536]), Err(HeapErrorKind::StaleReference));
    assert!(roots.iter().all(|id| heap.get(*id).is_ok()));
}

#[test]
fn one_hundred_thousand_overwritten_values_stay_within_fixed_live_payload_and_slot_budgets()
-> Result<(), String> {
    let mut heap = StringHeap::new(StringHeapBudget {
        max_live_strings: 64,
        max_slots: 64,
        max_payload_bytes: 1024,
        max_single_string_bytes: 16,
        ..StringHeapBudget::default()
    })
    .expect("Churn budget");
    let root = heap
        .allocate(b"keep", HeapWorkBudget::default())
        .expect("Persistent root")
        .id;
    let mut work = 0_u64;
    let mut collections = 0;
    for number in 0_u32..100_000 {
        let bytes = u128::from(number).to_le_bytes();
        let allocation = match heap.allocate(&bytes, HeapWorkBudget::default()) {
            Ok(allocation) => allocation,
            Err(error)
                if matches!(
                    error.kind,
                    HeapErrorKind::BudgetExceeded {
                        resource: HeapResource::LiveStrings,
                        ..
                    }
                ) =>
            {
                work += error.usage.steps;
                let collection = heap
                    .reclaim(&[root], HeapWorkBudget::default())
                    .expect("Complete remaining root");
                work += collection.usage.steps;
                collections += 1;
                heap.allocate(&bytes, HeapWorkBudget::default())
                    .expect("Vacant slot after collection")
            }
            Err(error) => return Err(format!("Unexpected churn failure: {error}")),
        };
        work += allocation.usage.steps;
        assert_eq!(heap.get(allocation.id).expect("Current temporary"), bytes);
        assert!(heap.usage().payload_bytes <= 1024);
        assert!(heap.usage().slots <= 64);
    }
    heap.reclaim(&[root], HeapWorkBudget::default())
        .expect("Final reclamation");
    assert_eq!(heap.usage().live_strings, 1);
    assert_eq!(heap.usage().payload_bytes, 4);
    assert_eq!(heap.usage().slots, 64);
    assert!(collections > 1000);
    assert!(work < 3_000_000);
    Ok(())
}
