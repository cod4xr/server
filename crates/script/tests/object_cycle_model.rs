// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Tests compare graph peeling with an independent owned-reference release model."
)]
use std::collections::HashSet;

use crate::object_heap_support::{allocate, stores};
use cod4x_script::execution::KernelValue;
use cod4x_script::heap::{HeapWorkBudget, ObjectHeapBudget};

#[test]
fn cycle_closure_and_live_boundary_match_4096_independent_reference_count_release_graphs() {
    let mut random = 0x6ab4_02d9_8001_746f_u64;
    for sample in 0..4096 {
        let count = usize::try_from(next(&mut random) % 13).expect("Small model");
        let mut graph = vec![Vec::new(); count];
        for edges in &mut graph {
            for _ in 0..next(&mut random) % 5 {
                edges.push(usize::try_from(next(&mut random) % count as u64).expect("Target"));
            }
        }
        let roots: Vec<_> = (0..count)
            .filter(|_| next(&mut random).is_multiple_of(4))
            .collect();
        let reachable = reachable(&graph, &roots);
        let released = release_model(&graph, &roots);
        let retained: Vec<_> = (0..count)
            .map(|index| !reachable[index] && !released[index])
            .collect();
        let boundary: Vec<_> = graph
            .iter()
            .enumerate()
            .filter(|(owner, _)| retained[*owner])
            .flat_map(|(_, edges)| edges.iter().copied())
            .filter(|target| reachable[*target])
            .collect();
        let (strings, mut heap) = stores(ObjectHeapBudget::default());
        let ids: Vec<_> = (0..count).map(|_| allocate(&mut heap)).collect();
        for (owner, edges) in graph.iter().enumerate() {
            for (number, target) in edges.iter().copied().enumerate() {
                heap.put(
                    ids[owner],
                    format!("edge{number}").as_bytes(),
                    KernelValue::Object(ids[target]),
                    &strings,
                    HeapWorkBudget::default(),
                )
                .expect("Modeled ownership edge");
            }
        }
        let roots: Vec<_> = roots.iter().map(|index| ids[*index]).collect();
        let report = heap
            .analyze_cycles(&roots, HeapWorkBudget::default())
            .expect("Independent topology");
        assert_eq!(
            report.unreachable_objects,
            reachable.iter().filter(|live| !**live).count() as u64,
            "sample {sample}"
        );
        assert_eq!(
            report.cycle_retained_objects,
            retained.iter().filter(|retained| **retained).count() as u64,
            "sample {sample}"
        );
        assert_eq!(
            report.cycle_to_live_edges,
            boundary.len() as u64,
            "sample {sample}"
        );
        let expected: HashSet<_> = boundary.iter().map(|target| ids[*target]).collect();
        assert_eq!(
            report
                .live_targets()
                .iter()
                .copied()
                .collect::<HashSet<_>>(),
            expected,
            "sample {sample}"
        );
    }
}

fn next(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    *state ^ (*state >> 29)
}

fn reachable(graph: &[Vec<usize>], roots: &[usize]) -> Vec<bool> {
    let mut seen = vec![false; graph.len()];
    let mut pending = roots.to_vec();
    while let Some(index) = pending.pop() {
        if !seen[index] {
            seen[index] = true;
            pending.extend_from_slice(&graph[index]);
        }
    }
    seen
}

fn release_model(graph: &[Vec<usize>], roots: &[usize]) -> Vec<bool> {
    // Each node begins with one creation owner. Field assignments add owners;
    // release every creation owner except supplied roots, cascading destruction.
    let mut owners = vec![1_usize; graph.len()];
    for target in graph.iter().flatten() {
        owners[*target] += 1;
    }
    let mut released = vec![false; graph.len()];
    let mut pending: Vec<_> = (0..graph.len())
        .filter(|index| !roots.contains(index))
        .collect();
    while let Some(index) = pending.pop() {
        assert!(owners[index] > 0);
        owners[index] -= 1;
        if owners[index] == 0 {
            assert!(!released[index]);
            released[index] = true;
            pending.extend_from_slice(&graph[index]);
        }
    }
    released
}
