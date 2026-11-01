// SPDX-License-Identifier: AGPL-3.0-or-later
//! Reproducible unique-entry storage; not populated COW, native VM timing or RSS.
use cod4x_script::execution::KernelValue;
use cod4x_script::heap::{
    ArrayKey, HeapWorkBudget, ObjectHeap, ObjectHeapBudget, StringHeap, StringHeapBudget,
};
use std::io::{self, Write};
use std::time::Instant;

type Failure = Box<dyn std::error::Error>;

fn wide(output: &mut impl Write) -> Result<(), Failure> {
    let strings = StringHeap::new(StringHeapBudget::default())?;
    let mut heap = ObjectHeap::new(&strings, ObjectHeapBudget::default())?;
    let budget = HeapWorkBudget::default();
    let value = heap.new_array_value(budget)?.value;
    let data = heap.array_data(value, budget)?.value;
    let started = Instant::now();
    let mut work = 0;
    for index in 0..70_000 {
        work += heap
            .array_put_unique(
                value,
                ArrayKey::Index(u64::MAX - index),
                KernelValue::Undefined,
                &strings,
                budget,
            )?
            .usage
            .steps;
    }
    let keys = heap.array_keys(data, budget)?;
    if keys.value.len() != 70_000 {
        return Err("Wide key count differs".into());
    }
    let usage = heap.usage();
    writeln!(
        output,
        "phase=wide entries={} retained_field_slots={} retained_index_buckets={} owned_encoded_key_bytes={} logical_heap_metadata_bytes={} insertion_work={} key_list_work={} key_list_scratch_bytes={} elapsed_us={}",
        usage.live_fields,
        usage.field_slots,
        usage.index_buckets,
        usage.key_bytes,
        usage.metadata_bytes,
        work,
        keys.usage.steps,
        keys.usage.scratch_bytes,
        started.elapsed().as_micros()
    )?;
    Ok(())
}

fn churn(output: &mut impl Write) -> Result<(), Failure> {
    let strings = StringHeap::new(StringHeapBudget::default())?;
    let mut heap = ObjectHeap::new(
        &strings,
        ObjectHeapBudget {
            max_live_fields: 1,
            max_field_slots: 1,
            max_key_bytes: 9,
            max_index_buckets: 8,
            ..ObjectHeapBudget::default()
        },
    )?;
    let budget = HeapWorkBudget::default();
    let value = heap.new_array_value(budget)?.value;
    let (mut work, mut scratch) = (0, 0);
    let started = Instant::now();
    for index in 0..100_000 {
        let key = ArrayKey::Index(u64::MAX - index);
        let insert = heap.array_put_unique(value, key, KernelValue::Undefined, &strings, budget)?;
        let replace =
            heap.array_put_unique(value, key, KernelValue::Undefined, &strings, budget)?;
        let remove = heap.array_remove_unique(value, key, &strings, budget)?;
        work += insert.usage.steps + replace.usage.steps + remove.usage.steps;
        scratch = scratch
            .max(insert.usage.scratch_bytes)
            .max(replace.usage.scratch_bytes)
            .max(remove.usage.scratch_bytes);
    }
    let usage = heap.usage();
    if usage.live_fields != 0 || usage.field_slots != 1 {
        return Err("Churn storage differs".into());
    }
    writeln!(
        output,
        "phase=churn cycles=100000 live_entries={} retained_field_slots={} retained_index_buckets={} owned_key_bytes={} logical_heap_metadata_bytes={} insert_replace_remove_work={} peak_scratch_bytes={} elapsed_us={}",
        usage.live_fields,
        usage.field_slots,
        usage.index_buckets,
        usage.key_bytes,
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
