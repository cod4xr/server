// SPDX-License-Identifier: AGPL-3.0-or-later
//! Current-value populated COW workload; not nested arrays, native VM timing or RSS.
use std::io::{self, Write};
use std::time::Instant;

use cod4x_script::execution::KernelValue;
use cod4x_script::heap::{
    ArrayCowOutcome, ArrayKey, HeapWorkBudget, ObjectHeap, ObjectHeapBudget, StringHeap,
    StringHeapBudget,
};

type Failure = Box<dyn std::error::Error>;

fn wide(output: &mut impl Write) -> Result<(), Failure> {
    let strings = StringHeap::new(StringHeapBudget::default())?;
    let mut heap = ObjectHeap::new(&strings, ObjectHeapBudget::default())?;
    let budget = HeapWorkBudget::default();
    let source = heap.new_array_value(budget)?.value;
    for index in 0..70_000 {
        heap.array_put_unique(
            source,
            ArrayKey::Index(index),
            KernelValue::Undefined,
            &strings,
            budget,
        )?;
    }
    let alias = heap.copy_array_value(source, budget)?.value;
    let started = Instant::now();
    let copied = heap.make_array_unique(alias, &strings, budget)?;
    let elapsed = started.elapsed().as_micros();
    let data = heap.array_data(alias, budget)?.value;
    if copied.value != ArrayCowOutcome::Detached || heap.array_len(data, budget)?.value != 70_000 {
        return Err("Wide copy result differs".into());
    }
    let usage = heap.usage();
    writeln!(
        output,
        "phase=wide copied_entries=70000 live_fields={} retained_field_slots={} retained_index_buckets={} owned_encoded_key_bytes={} logical_heap_metadata_bytes={} copy_work={} copy_scratch_bytes={} elapsed_us={}",
        usage.live_fields,
        usage.field_slots,
        usage.index_buckets,
        usage.key_bytes,
        usage.metadata_bytes,
        copied.usage.steps,
        copied.usage.scratch_bytes,
        elapsed
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
            max_live_fields: 8,
            max_field_slots: 8,
            max_key_bytes: 72,
            max_index_buckets: 16,
            ..ObjectHeapBudget::default()
        },
    )?;
    let budget = HeapWorkBudget::default();
    let source = heap.new_array_value(budget)?.value;
    for index in 0..4 {
        heap.array_put_unique(
            source,
            ArrayKey::Index(index),
            KernelValue::Undefined,
            &strings,
            budget,
        )?;
    }
    let (mut copy_work, mut maintenance_work, mut scratch) = (0, 0, 0);
    let started = Instant::now();
    for _ in 0..100_000 {
        let alias = heap.copy_array_value(source, budget)?;
        let copied = heap.make_array_unique(alias.value, &strings, budget)?;
        let release = heap.release_array_value(alias.value, budget)?;
        let collection = heap.reclaim(&[], budget)?;
        if copied.value != ArrayCowOutcome::Detached
            || collection.objects != 1
            || collection.fields != 4
        {
            return Err("Churn copy result differs".into());
        }
        copy_work += copied.usage.steps;
        maintenance_work += alias.usage.steps + release.usage.steps + collection.usage.steps;
        scratch = scratch
            .max(copied.usage.scratch_bytes)
            .max(collection.usage.scratch_bytes);
    }
    let usage = heap.usage();
    writeln!(
        output,
        "phase=churn cycles=100000 live_fields={} retained_object_slots={} retained_value_slots={} retained_field_slots={} retained_index_buckets={} owned_key_bytes={} logical_heap_metadata_bytes={} copy_work={} share_release_collect_work={} peak_operation_scratch_bytes={} elapsed_us={}",
        usage.live_fields,
        usage.object_slots,
        usage.array_value_slots,
        usage.field_slots,
        usage.index_buckets,
        usage.key_bytes,
        usage.metadata_bytes,
        copy_work,
        maintenance_work,
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
