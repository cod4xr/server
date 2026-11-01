// SPDX-License-Identifier: AGPL-3.0-or-later
//! Managed empty-array ownership workload; not native VM/GC performance or RSS.
use std::io::{self, Write};
use std::time::Instant;

use cod4x_script::heap::{
    ArrayCowOutcome, HeapWorkBudget, ObjectHeap, ObjectHeapBudget, StringHeap, StringHeapBudget,
};

type Failure = Box<dyn std::error::Error>;

fn wide(output: &mut impl Write) -> Result<(), Failure> {
    let strings = StringHeap::new(StringHeapBudget::default())?;
    let mut heap = ObjectHeap::new(&strings, ObjectHeapBudget::default())?;
    let budget = HeapWorkBudget::default();
    let value = heap.new_array_value(budget)?.value;
    let mut work = 0;
    let started = Instant::now();
    for _ in 1..70_000 {
        // These locations intentionally stay host-owned until the heap is dropped.
        work += heap.copy_array_value(value, budget)?.usage.steps;
    }
    let data = heap.array_data(value, budget)?.value;
    if heap.array_owner_count(data, budget)?.value != 70_000 {
        return Err("Wide owner count differs".into());
    }
    work += heap.make_empty_array_unique(value, budget)?.usage.steps;
    let usage = heap.usage();
    writeln!(
        output,
        "phase=wide owned_values={} data_nodes={} retained_value_slots={} metadata_rows={} logical_heap_metadata_bytes={} copy_detach_work={} elapsed_us={}",
        usage.live_array_values,
        usage.live_arrays,
        usage.array_value_slots,
        usage.array_metadata_rows,
        usage.metadata_bytes,
        work,
        started.elapsed().as_micros()
    )?;
    Ok(())
}

fn churn(output: &mut impl Write) -> Result<(), Failure> {
    let strings = StringHeap::new(StringHeapBudget::default())?;
    let mut heap = ObjectHeap::new(
        &strings,
        ObjectHeapBudget {
            max_live_objects: 2,
            max_object_slots: 2,
            max_array_values: 2,
            max_array_value_slots: 2,
            ..ObjectHeapBudget::default()
        },
    )?;
    let budget = HeapWorkBudget::default();
    let root = heap.new_array_value(budget)?.value;
    let (mut work, mut scratch) = (0, 0);
    let started = Instant::now();
    for _ in 0..100_000 {
        let copy = heap.copy_array_value(root, budget)?;
        work += copy.usage.steps;
        let detach = heap.make_empty_array_unique(copy.value, budget)?;
        if detach.value != ArrayCowOutcome::Detached {
            return Err("Shared value failed to detach".into());
        }
        work += detach.usage.steps;
        work += heap.release_array_value(copy.value, budget)?.usage.steps;
        let collection = heap.reclaim(&[], budget)?;
        if collection.objects != 1 {
            return Err("Detached data was not reclaimed".into());
        }
        work += collection.usage.steps;
        scratch = scratch.max(collection.usage.scratch_bytes);
    }
    let usage = heap.usage();
    writeln!(
        output,
        "phase=churn cycles=100000 live_values={} data_nodes={} retained_object_slots={} retained_value_slots={} metadata_rows={} logical_heap_metadata_bytes={} copy_detach_release_collect_work={} peak_scratch_bytes={} elapsed_us={}",
        usage.live_array_values,
        usage.live_arrays,
        usage.object_slots,
        usage.array_value_slots,
        usage.array_metadata_rows,
        usage.metadata_bytes,
        work,
        scratch,
        started.elapsed().as_micros()
    )?;
    Ok(())
}

fn main() -> Result<(), Failure> {
    let mut output = io::BufWriter::new(io::stdout().lock());
    wide(&mut output)?;
    churn(&mut output)?;
    Ok(())
}
