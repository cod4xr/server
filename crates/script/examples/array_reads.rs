// SPDX-License-Identifier: AGPL-3.0-or-later
//! Compiled source reads a host-managed array; this is not native whole-VM parity.
use std::io::{self, Write};

use cod4x_script::binding::{LegacyDeveloperMode, LegacyModuleSymbols, SymbolBudget};
use cod4x_script::compilation::{self, CompiledModule, ModuleCompilationBudget};
use cod4x_script::declaration::{DeclarationBudget, LegacyModuleSyntax};
use cod4x_script::execution::{KernelCallBudget, KernelHeapContext, KernelValue};
use cod4x_script::heap::{
    ArrayKey, HeapWorkBudget, ObjectHeap, ObjectHeapBudget, StringHeap, StringHeapBudget,
};
use cod4x_script::legacy::LegacyNumber;
use cod4x_script::source::{SourceBudget, SourceText};
use cod4x_script::syntax::{LegacySyntaxBudget, LegacySyntaxMode, LegacySyntaxTree};

type Failure = Box<dyn std::error::Error>;

fn compile() -> Result<CompiledModule, Failure> {
    let source = SourceText::index(
        include_bytes!("../../../examples/gsc/array_reads.gsc"),
        SourceBudget::default(),
    )?;
    let tree = LegacySyntaxTree::parse_structure(
        &source,
        LegacySyntaxMode::Script,
        LegacySyntaxBudget::default(),
    )?;
    let module = LegacyModuleSyntax::lower(&tree, DeclarationBudget::default())?;
    let symbols = LegacyModuleSymbols::bind(
        &module,
        LegacyDeveloperMode::Disabled,
        SymbolBudget::default(),
    )?;
    Ok(compilation::compile_bound_primitive_module(
        &symbols,
        ModuleCompilationBudget::default(),
    )?)
}

fn main() -> Result<(), Failure> {
    let module = compile()?;
    let work = HeapWorkBudget::default();
    let mut strings = StringHeap::new(StringHeapBudget::default())?;
    let mut objects = ObjectHeap::new(&strings, ObjectHeapBudget::default())?;
    let host = objects.new_array_value(work)?.value;
    let data = objects.array_data(host, work)?.value;
    for (key, value) in [(ArrayKey::Index(0), 7), (ArrayKey::Bytes(b"bonus"), 11)] {
        objects.array_put_unique(
            host,
            key,
            KernelValue::Number(LegacyNumber::Integer(value)),
            &strings,
            work,
        )?;
    }
    let mut context =
        KernelHeapContext::new(&mut strings, &[], work).with_objects(&mut objects, &[]);
    let result = module.program().execute_with_heap(
        0,
        &[KernelValue::Array(host)],
        KernelCallBudget::default(),
        &mut context,
    )?;
    if result.value != KernelValue::Number(LegacyNumber::Integer(20)) {
        return Err("Array read example result differs".into());
    }
    let heap = context.usage();
    let usage = context
        .object_heap_usage()
        .ok_or("Missing object context")?;
    drop(context);
    if objects.array_owner_count(data, work)?.value != 1 || usage.live_array_values != 1 {
        return Err("Array read example retained a VM owner".into());
    }
    let mut output = io::BufWriter::new(io::stdout().lock());
    writeln!(
        output,
        "result=20 functions={} vm_steps={} peak_frames={} heap_work={} peak_context_scratch_bytes={} live_fields={} live_array_values={} retained_value_slots={}",
        module.program().usage().functions,
        result.usage.steps,
        result.usage.peak_frames,
        heap.steps,
        heap.peak_scratch_bytes,
        usage.live_fields,
        usage.live_array_values,
        usage.array_value_slots
    )?;
    output.flush()?;
    Ok(())
}
