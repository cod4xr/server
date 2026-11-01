// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Deterministic bounded operation sequences exercise checked store transitions."
)]

use cod4x_script::heap::{HeapErrorKind, HeapWorkBudget, StringHeap, StringHeapBudget, StringId};

#[test]
fn hostile_allocation_concat_root_and_lookup_sequences_preserve_usage_and_exact_live_bytes()
-> Result<(), String> {
    let mut heap = StringHeap::new(StringHeapBudget {
        max_live_strings: 32,
        max_slots: 64,
        max_payload_bytes: 512,
        max_single_string_bytes: 32,
        ..StringHeapBudget::default()
    })
    .expect("Store");
    let mut foreign = StringHeap::new(StringHeapBudget::default()).expect("Foreign store");
    let foreign = foreign
        .allocate(b"foreign", HeapWorkBudget::default())
        .expect("Foreign value")
        .id;
    let mut history: Vec<(StringId, Vec<u8>)> = Vec::new();
    let mut state = 0x5a7c_d149_u64;
    let work = HeapWorkBudget {
        max_steps: 512,
        max_scratch_bytes: 64,
    };
    for _ in 0..4096 {
        let random = next(&mut state);
        let before = heap.usage();
        match random % 4 {
            0 => {
                let bytes = vec![
                    random.to_le_bytes()[0];
                    usize::try_from(random % 33).expect("Small payload")
                ];
                match heap.allocate(&bytes, work) {
                    Ok(allocation) => history.push((allocation.id, bytes)),
                    Err(error) if matches!(error.kind, HeapErrorKind::BudgetExceeded { .. }) => {
                        assert_eq!(heap.usage(), before);
                    }
                    Err(error) => return Err(format!("Unexpected allocation: {error}")),
                }
            }
            1 => {
                let mut roots: Vec<_> = history
                    .iter()
                    .filter(|(id, _)| heap.get(*id).is_ok())
                    .enumerate()
                    .filter(|(index, _)| index % 2 == 0)
                    .map(|(_, (id, _))| *id)
                    .collect();
                if random & 16 != 0 {
                    roots.push(foreign);
                }
                match heap.reclaim(&roots, work) {
                    Ok(collection) => {
                        assert_eq!(
                            collection.freed_strings,
                            before.live_strings - heap.usage().live_strings
                        );
                        assert_eq!(
                            collection.freed_payload_bytes,
                            before.payload_bytes - heap.usage().payload_bytes
                        );
                    }
                    Err(error) if error.kind == HeapErrorKind::ForeignReference => {
                        assert_eq!(heap.usage(), before);
                    }
                    Err(error) => return Err(format!("Unexpected collection: {error}")),
                }
            }
            2 if !history.is_empty() => {
                let index = usize::try_from(random % history.len() as u64).expect("History index");
                let (id, bytes) = &history[index];
                match heap.get(*id) {
                    Ok(value) => assert_eq!(value, bytes),
                    Err(HeapErrorKind::StaleReference) => {}
                    Err(error) => return Err(format!("Unexpected lookup: {error:?}")),
                }
            }
            3 if !history.is_empty() => {
                let index = usize::try_from(random % history.len() as u64).expect("History index");
                let (id, bytes) = &history[index];
                let expected = [bytes.as_slice(), bytes.as_slice()].concat();
                match heap.concatenate(*id, *id, work) {
                    Ok(allocation) => history.push((allocation.id, expected)),
                    Err(error)
                        if matches!(
                            error.kind,
                            HeapErrorKind::StaleReference | HeapErrorKind::BudgetExceeded { .. }
                        ) =>
                    {
                        assert_eq!(heap.usage(), before);
                    }
                    Err(error) => return Err(format!("Unexpected concatenation: {error}")),
                }
            }
            _ => {}
        }
        verify_usage(&heap, &history)?;
    }
    Ok(())
}

fn verify_usage(heap: &StringHeap, history: &[(StringId, Vec<u8>)]) -> Result<(), String> {
    let (mut live, mut payload) = (0, 0);
    for (id, expected) in history {
        match heap.get(*id) {
            Ok(bytes) => {
                assert_eq!(bytes, expected);
                live += 1;
                payload += bytes.len() as u64;
            }
            Err(HeapErrorKind::StaleReference) => {}
            Err(error) => return Err(format!("Unexpected history reference: {error}")),
        }
    }
    assert_eq!(heap.usage().live_strings, live);
    assert_eq!(heap.usage().payload_bytes, payload);
    assert!(live <= 32 && payload <= 512 && heap.usage().slots <= 64);
    Ok(())
}

fn next(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}
