// SPDX-License-Identifier: AGPL-3.0-or-later
//! Explicit ordinary root bindings, not an implicit game world or native VM parity.
use std::io::{self, Write};

use cod4x_script::binding::{LegacyDeveloperMode, LegacyModuleSymbols, SymbolBudget};
use cod4x_script::compilation::{self, CompiledModule, ModuleCompilationBudget};
use cod4x_script::declaration::{DeclarationBudget, LegacyModuleSyntax};
use cod4x_script::execution::{
    KernelCallBudget, KernelHeapContext, KernelScriptRoots, KernelValue,
};
use cod4x_script::heap::{
    HeapWorkBudget, ObjectHeap, ObjectHeapBudget, StringHeap, StringHeapBudget,
};
use cod4x_script::legacy::LegacyNumber;
use cod4x_script::source::{SourceBudget, SourceText};
use cod4x_script::syntax::{LegacySyntaxBudget, LegacySyntaxMode, LegacySyntaxTree};

type Failure = Box<dyn std::error::Error>;

fn compile() -> Result<CompiledModule, Failure> {
    let source = SourceText::index(
        include_bytes!("../../../examples/gsc/roots.gsc"),
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
    let level = objects.allocate(work)?.id;
    let receiver = objects.allocate(work)?.id;
    let animation = objects.allocate(work)?.id;
    let game = objects.new_array_value(work)?.value;
    let roots = KernelScriptRoots::new(level, receiver, game, animation);
    let mut context = KernelHeapContext::new(&mut strings, &[], work)
        .with_objects(&mut objects, &[])
        .with_script_roots(roots);
    let result =
        module
            .program()
            .execute_with_heap(0, &[], KernelCallBudget::default(), &mut context)?;
    if result.value != KernelValue::Number(LegacyNumber::Integer(61)) {
        return Err("Root example result differs".into());
    }
    let heap = context.usage();
    let objects = context
        .object_heap_usage()
        .ok_or("Missing object context")?;
    let mut output = io::BufWriter::new(io::stdout().lock());
    writeln!(
        output,
        "result=61 functions={} vm_steps={} peak_frames={} heap_work={} peak_context_scratch_bytes={} live_fields={} live_array_values={} retained_value_slots={}",
        module.program().usage().functions,
        result.usage.steps,
        result.usage.peak_frames,
        heap.steps,
        heap.peak_scratch_bytes,
        objects.live_fields,
        objects.live_array_values,
        objects.array_value_slots
    )?;
    output.flush()?;
    Ok(())
}
