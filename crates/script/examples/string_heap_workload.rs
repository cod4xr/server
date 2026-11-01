// SPDX-License-Identifier: AGPL-3.0-or-later
//! Measure owned leaf-string churn; this is not a full VM/native performance benchmark.

use std::io::{self, Write};
use std::time::Instant;

use cod4x_script::heap::{
    HeapErrorKind, HeapResource, HeapWorkBudget, StringHeap, StringHeapBudget,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let started = Instant::now();
    let mut heap = StringHeap::new(StringHeapBudget {
        max_live_strings: 64,
        max_slots: 64,
        max_payload_bytes: 1024,
        max_single_string_bytes: 16,
        ..StringHeapBudget::default()
    })?;
    let root = heap.allocate(b"keep", HeapWorkBudget::default())?.id;
    let mut work = 0_u64;
    let mut collections = 0_u64;
    let mut peak_payload = heap.usage().payload_bytes;
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
                work += heap
                    .reclaim(&[root], HeapWorkBudget::default())?
                    .usage
                    .steps;
                collections += 1;
                heap.allocate(&bytes, HeapWorkBudget::default())?
            }
            Err(error) => return Err(error.into()),
        };
        work += allocation.usage.steps;
        peak_payload = peak_payload.max(heap.usage().payload_bytes);
        if heap.get(allocation.id)? != bytes {
            return Err("Measured payload differs".into());
        }
    }
    work += heap
        .reclaim(&[root], HeapWorkBudget::default())?
        .usage
        .steps;
    let usage = heap.usage();
    let mut output = io::stdout().lock();
    writeln!(
        output,
        "operations=100000 collections={collections} logical_work={work} peak_payload_bytes={peak_payload} retained_payload_bytes={} retained_slots={} logical_metadata_bytes={} elapsed_us={}",
        usage.payload_bytes,
        usage.slots,
        usage.metadata_bytes,
        started.elapsed().as_micros()
    )?;
    output.flush()?;
    Ok(())
}
