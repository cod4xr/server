// SPDX-License-Identifier: AGPL-3.0-or-later
//! Measure explicit shared-graph/string churn, not native VM performance or RSS.
use std::io::{self, Write};
use std::time::Instant;

use cod4x_script::execution::KernelValue;
use cod4x_script::heap::{
    HeapWorkBudget, HeapWorkUsage, ObjectHeap, ObjectHeapBudget, ObjectHeapErrorKind, ObjectId,
    ObjectResource, StringHeap, StringHeapBudget,
};

#[derive(Default)]
struct Measurement {
    work: u64,
    scratch: u64,
    collections: u64,
    peak_key_bytes: u64,
    peak_string_bytes: u64,
    peak_metadata: u64,
}

impl Measurement {
    fn record(&mut self, usage: HeapWorkUsage) {
        self.work += usage.steps;
        self.scratch = self.scratch.max(usage.scratch_bytes);
    }

    fn peak(&mut self, objects: &ObjectHeap, strings: &StringHeap) {
        self.peak_key_bytes = self.peak_key_bytes.max(objects.usage().key_bytes);
        self.peak_string_bytes = self.peak_string_bytes.max(strings.usage().payload_bytes);
        self.peak_metadata = self
            .peak_metadata
            .max(objects.usage().metadata_bytes + strings.usage().metadata_bytes);
    }

    fn collect(
        &mut self,
        objects: &mut ObjectHeap,
        strings: &mut StringHeap,
        root: ObjectId,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.record(objects.reclaim(&[root], HeapWorkBudget::default())?.usage);
        self.record(
            objects
                .reclaim_strings(strings, &[], HeapWorkBudget::default())?
                .usage,
        );
        self.collections += 1;
        Ok(())
    }
}

fn stores() -> Result<(StringHeap, ObjectHeap), Box<dyn std::error::Error>> {
    let strings = StringHeap::new(StringHeapBudget {
        max_live_strings: 64,
        max_slots: 64,
        max_payload_bytes: 1024,
        max_single_string_bytes: 16,
        ..StringHeapBudget::default()
    })?;
    let objects = ObjectHeap::new(
        &strings,
        ObjectHeapBudget {
            max_live_objects: 64,
            max_object_slots: 64,
            max_live_fields: 128,
            max_field_slots: 128,
            max_index_buckets: 512,
            max_key_bytes: 1024,
            ..ObjectHeapBudget::default()
        },
    )?;
    Ok((strings, objects))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let started = Instant::now();
    let work = HeapWorkBudget::default();
    let (mut strings, mut objects) = stores()?;
    let mut measured = Measurement::default();
    let allocation = objects.allocate(work)?;
    measured.record(allocation.usage);
    let root = allocation.id;
    let label = strings.allocate(b"keep", work)?;
    measured.record(label.usage);
    measured.record(
        objects
            .put(
                root,
                b"label",
                KernelValue::String(label.id),
                &strings,
                work,
            )?
            .usage,
    );
    for number in 0_u64..100_000 {
        let allocation = match objects.allocate(work) {
            Ok(result) => result,
            Err(error)
                if matches!(
                    error.kind,
                    ObjectHeapErrorKind::BudgetExceeded {
                        resource: ObjectResource::LiveObjects,
                        ..
                    }
                ) =>
            {
                measured.record(error.usage);
                measured.collect(&mut objects, &mut strings, root)?;
                objects.allocate(work)?
            }
            Err(error) => return Err(error.into()),
        };
        measured.record(allocation.usage);
        let id = allocation.id;
        let payload = u128::from(number).to_le_bytes();
        let label = strings.allocate(&payload, work)?;
        measured.record(label.usage);
        measured.record(
            objects
                .put(
                    id,
                    b"label",
                    KernelValue::LocalizedString(label.id),
                    &strings,
                    work,
                )?
                .usage,
        );
        measured.record(
            objects
                .put(id, b"back", KernelValue::Object(root), &strings, work)?
                .usage,
        );
        measured.record(
            objects
                .put(root, b"head", KernelValue::Object(id), &strings, work)?
                .usage,
        );
        if strings.get(label.id)? != payload {
            return Err("Measured payload differs".into());
        }
        measured.peak(&objects, &strings);
    }
    measured.record(objects.remove(root, b"head", &strings, work)?.usage);
    measured.collect(&mut objects, &mut strings, root)?;
    let usage = objects.usage();
    let mut output = io::stdout().lock();
    writeln!(
        output,
        "operations=100000 collection_rounds={} logical_work={} peak_scratch_bytes={} peak_key_bytes={} peak_string_bytes={} peak_logical_metadata_bytes={} retained_objects={} retained_fields={} retained_key_bytes={} retained_string_bytes={} retained_object_slots={} retained_field_slots={} retained_index_buckets={} retained_string_slots={} elapsed_us={}",
        measured.collections,
        measured.work,
        measured.scratch,
        measured.peak_key_bytes,
        measured.peak_string_bytes,
        measured.peak_metadata,
        usage.live_objects,
        usage.live_fields,
        usage.key_bytes,
        strings.usage().payload_bytes,
        usage.object_slots,
        usage.field_slots,
        usage.index_buckets,
        strings.usage().slots,
        started.elapsed().as_micros()
    )?;
    output.flush()?;
    Ok(())
}
