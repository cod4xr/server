// SPDX-License-Identifier: AGPL-3.0-or-later
//! Managed VM transport measurements, not source arrays, native VM timing or RSS.
use std::io::{self, Write};
use std::time::Instant;

use cod4x_script::execution::{
    KernelBuiltin, KernelExecutionBudget, KernelFunctionLayout, KernelHeapContext,
    KernelInstruction, KernelValue, KernelVerificationBudget, VerifiedKernelProgram,
};
use cod4x_script::heap::{
    HeapWorkBudget, ObjectHeap, ObjectHeapBudget, StringHeap, StringHeapBudget,
};

type Failure = Box<dyn std::error::Error>;

#[derive(Clone, Copy)]
struct Profile {
    name: &'static str,
    locals: u64,
    unrelated: u64,
    heap: HeapWorkBudget,
}

fn run(
    output: &mut impl Write,
    code: &[KernelInstruction],
    profile: Profile,
) -> Result<(), Failure> {
    let proof = VerifiedKernelProgram::verify_function(
        code,
        KernelFunctionLayout::new(1, profile.locals)?,
        KernelVerificationBudget::default(),
    )?;
    let mut strings = StringHeap::new(StringHeapBudget::default())?;
    let slots = profile.unrelated + profile.locals + 2;
    let mut objects = ObjectHeap::new(
        &strings,
        ObjectHeapBudget {
            max_array_values: slots,
            max_array_value_slots: slots,
            ..ObjectHeapBudget::default()
        },
    )?;
    let host = objects.new_array_value(HeapWorkBudget::default())?.value;
    for _ in 0..profile.unrelated {
        objects.copy_array_value(host, HeapWorkBudget::default())?;
    }
    let mut context =
        KernelHeapContext::new(&mut strings, &[], profile.heap).with_objects(&mut objects, &[]);
    let started = Instant::now();
    let result = proof.execute_with_heap(
        &[KernelValue::Array(host)],
        KernelExecutionBudget::default(),
        &mut context,
    )?;
    let elapsed = started.elapsed().as_micros();
    let heap = context.usage();
    let objects = context
        .object_heap_usage()
        .ok_or("Missing object context")?;
    if objects.live_array_values != profile.unrelated + 1 || objects.array_value_slots != slots {
        return Err("Managed owner/slot count differs".into());
    }
    writeln!(
        output,
        "phase={} unrelated_host_aliases={} local_values={} vm_steps={} heap_work_including_prepaid_cleanup={} peak_context_scratch_bytes={} retained_value_slots={} live_values_after_cleanup={} elapsed_us={}",
        profile.name,
        profile.unrelated,
        result.local_values,
        result.steps,
        heap.steps,
        heap.peak_scratch_bytes,
        objects.array_value_slots,
        objects.live_array_values,
        elapsed
    )?;
    Ok(())
}

fn main() -> Result<(), Failure> {
    let mut output = io::BufWriter::new(io::stdout().lock());
    let mut wide = Vec::new();
    wide.try_reserve_exact(140_003)?;
    for slot in 1..=70_000 {
        wide.extend([
            KernelInstruction::LoadLocal(0),
            KernelInstruction::StoreLocal(slot),
        ]);
    }
    wide.extend([
        KernelInstruction::LoadLocal(70_000),
        KernelInstruction::Builtin {
            function: KernelBuiltin::IsDefined,
            arguments: 1,
        },
        KernelInstruction::Return,
    ]);
    run(
        &mut output,
        &wide,
        Profile {
            name: "wide",
            locals: 70_001,
            unrelated: 0,
            heap: HeapWorkBudget::default(),
        },
    )?;
    drop(wide);
    let mut churn = Vec::new();
    churn.try_reserve_exact(200_067)?;
    for slot in 1..=32 {
        churn.extend([
            KernelInstruction::LoadLocal(0),
            KernelInstruction::StoreLocal(slot),
        ]);
    }
    for _ in 0..100_000 {
        churn.extend([
            KernelInstruction::LoadLocal(0),
            KernelInstruction::StoreLocal(33),
        ]);
    }
    churn.extend([
        KernelInstruction::LoadLocal(33),
        KernelInstruction::Builtin {
            function: KernelBuiltin::IsDefined,
            arguments: 1,
        },
        KernelInstruction::Return,
    ]);
    run(
        &mut output,
        &churn,
        Profile {
            name: "churn",
            locals: 34,
            unrelated: 70_000,
            heap: HeapWorkBudget {
                max_scratch_bytes: 4096,
                ..HeapWorkBudget::default()
            },
        },
    )?;
    output.flush()?;
    Ok(())
}
