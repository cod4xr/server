// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests prevent repeated small copies from scanning unrelated heap/index contents."
)]
use crate::object_heap_support::{allocate, number, stores};
use cod4x_script::heap::{ArrayKey, HeapWorkBudget, ObjectHeapBudget};

#[test]
fn repeated_tiny_copies_reuse_tombstone_positions_at_half_fill_without_global_rehash() {
    let (strings, mut heap) = stores(ObjectHeapBudget::default());
    let budget = HeapWorkBudget::default();
    let source = heap.new_array_value(budget).expect("Source").value;
    for index in 0..2 {
        heap.array_put_unique(source, ArrayKey::Index(index), number(7), &strings, budget)
            .expect("Tiny source");
    }
    let unrelated = allocate(&mut heap);
    for index in 0..65_532_u64 {
        heap.put(unrelated, &index.to_le_bytes(), number(9), &strings, budget)
            .expect("Unrelated heap/index contents");
    }
    let alias = heap
        .copy_array_value(source, budget)
        .expect("Warm alias")
        .value;
    heap.make_array_unique(alias, &strings, budget)
        .expect("Fill index exactly halfway");
    assert_eq!(heap.usage().live_fields, 65_536);
    assert_eq!(heap.usage().index_buckets, 131_072);
    heap.release_array_value(alias, budget)
        .expect("Release warm copy");
    heap.reclaim(&[unrelated], budget)
        .expect("Leave reusable bucket tombstones");
    for _ in 0..8 {
        let alias = heap.copy_array_value(source, budget).expect("Alias").value;
        let copied = heap
            .make_array_unique(
                alias,
                &strings,
                HeapWorkBudget {
                    max_steps: 1024,
                    max_scratch_bytes: 4096,
                },
            )
            .expect("Two-entry copy must not scan/clone the 65k unrelated fields/index");
        assert!(copied.usage.steps <= 1024 && copied.usage.scratch_bytes <= 4096);
        heap.release_array_value(alias, budget)
            .expect("Release copy");
        heap.reclaim(&[unrelated], budget)
            .expect("Reuse identical parent/key bucket positions");
    }
    assert_eq!(heap.field_count(unrelated), Ok(65_532));
}
