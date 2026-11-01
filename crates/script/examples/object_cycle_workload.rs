// SPDX-License-Identifier: AGPL-3.0-or-later
//! Reproducible graph ownership analysis; not Rust arrays, native VM performance or RSS.
use std::io::{self, Write};
use std::time::Instant;

use cod4x_script::execution::KernelValue;
use cod4x_script::heap::{
    HeapWorkBudget, ObjectHeap, ObjectHeapBudget, StringHeap, StringHeapBudget,
};

type Failure = Box<dyn std::error::Error>;

fn wide(output: &mut impl Write) -> Result<(), Failure> {
    let strings = StringHeap::new(StringHeapBudget::default())?;
    let mut heap = ObjectHeap::new(&strings, ObjectHeapBudget::default())?;
    let budget = HeapWorkBudget::default();
    let root = heap.allocate(budget)?.id;
    let first = heap.allocate(budget)?.id;
    let mut previous = first;
    for _ in 1..70_000 {
        let next = heap.allocate(budget)?.id;
        heap.put(
            previous,
            b"next",
            KernelValue::Object(next),
            &strings,
            budget,
        )?;
        previous = next;
    }
    heap.put(
        previous,
        b"next",
        KernelValue::Object(first),
        &strings,
        budget,
    )?;
    heap.put(
        previous,
        b"live",
        KernelValue::Object(root),
        &strings,
        budget,
    )?;
    let started = Instant::now();
    let analysis = heap.analyze_cycles(&[root], budget)?;
    if analysis.cycle_retained_objects != 70_000 || analysis.live_targets() != [root] {
        return Err("Wide graph result differs".into());
    }
    writeln!(
        output,
        "phase=wide unreachable={} cycle_closure={} live_edges={} unique_targets={} work={} peak_scratch_bytes={} logical_heap_metadata_bytes={} elapsed_us={}",
        analysis.unreachable_objects,
        analysis.cycle_retained_objects,
        analysis.cycle_to_live_edges,
        analysis.live_targets().len(),
        analysis.usage.steps,
        analysis.usage.scratch_bytes,
        heap.usage().metadata_bytes,
        started.elapsed().as_micros()
    )?;
    Ok(())
}

fn churn(output: &mut impl Write) -> Result<(), Failure> {
    let strings = StringHeap::new(StringHeapBudget::default())?;
    let mut heap = ObjectHeap::new(
        &strings,
        ObjectHeapBudget {
            max_live_objects: 65,
            max_object_slots: 65,
            max_live_fields: 128,
            max_field_slots: 128,
            ..ObjectHeapBudget::default()
        },
    )?;
    let budget = HeapWorkBudget::default();
    let root = heap.allocate(budget)?.id;
    let (mut remaining, mut rounds, mut work, mut scratch) = (100_000_u64, 0, 0, 0);
    let started = Instant::now();
    while remaining > 0 {
        let count = remaining.min(64);
        for _ in 0..count {
            let cycle = heap.allocate(budget)?.id;
            heap.put(cycle, b"self", KernelValue::Object(cycle), &strings, budget)?;
            heap.put(cycle, b"live", KernelValue::Object(root), &strings, budget)?;
        }
        let analysis = heap.analyze_cycles(&[root], budget)?;
        if analysis.cycle_retained_objects != count
            || analysis.cycle_to_live_edges != count
            || analysis.live_targets() != [root]
        {
            return Err("Churn graph result differs".into());
        }
        work += analysis.usage.steps;
        scratch = scratch.max(analysis.usage.scratch_bytes);
        heap.reclaim(&[root], budget)?;
        remaining -= count;
        rounds += 1;
    }
    writeln!(
        output,
        "phase=churn cycles=100000 rounds={rounds} analysis_work={work} analysis_peak_scratch_bytes={scratch} retained_objects={} retained_object_slots={} retained_field_slots={} elapsed_us={}",
        heap.usage().live_objects,
        heap.usage().object_slots,
        heap.usage().field_slots,
        started.elapsed().as_micros()
    )?;
    Ok(())
}

fn main() -> Result<(), Failure> {
    let mut output = io::stdout().lock();
    wide(&mut output)?;
    churn(&mut output)?;
    output.flush()?;
    Ok(())
}
