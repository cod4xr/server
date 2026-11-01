// SPDX-License-Identifier: AGPL-3.0-or-later
//! Closed supplied source files; no filesystem loader or native VM parity is implied.
use std::io::{self, Write};

use cod4x_script::binding::{LegacyDeveloperMode, LegacyModuleSymbols, SymbolBudget};
use cod4x_script::compilation::{
    self, CompiledScriptPackage, ScriptModuleDefinition, ScriptPackageBudget,
};
use cod4x_script::declaration::{DeclarationBudget, LegacyModuleSyntax};
use cod4x_script::execution::{KernelCallBudget, KernelHeapContext, KernelValue};
use cod4x_script::heap::{
    HeapWorkBudget, ObjectHeap, ObjectHeapBudget, StringHeap, StringHeapBudget,
};
use cod4x_script::legacy::LegacyNumber;
use cod4x_script::source::{SourceBudget, SourceText};
use cod4x_script::syntax::{LegacySyntaxBudget, LegacySyntaxMode, LegacySyntaxTree};

type Failure = Box<dyn std::error::Error>;
const INPUTS: [(&[u8], &[u8]); 2] = [
    (
        b"demo/root",
        include_bytes!("../../../examples/gsc/package/root.gsc"),
    ),
    (
        b"demo/library",
        include_bytes!("../../../examples/gsc/package/library.gsc"),
    ),
];

fn compile() -> Result<CompiledScriptPackage, Failure> {
    let sources = INPUTS
        .iter()
        .map(|(_, source)| SourceText::index(source, SourceBudget::default()))
        .collect::<Result<Vec<_>, _>>()?;
    let trees = sources
        .iter()
        .map(|source| {
            LegacySyntaxTree::parse_structure(
                source,
                LegacySyntaxMode::Script,
                LegacySyntaxBudget::default(),
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let modules = trees
        .iter()
        .map(|tree| LegacyModuleSyntax::lower(tree, DeclarationBudget::default()))
        .collect::<Result<Vec<_>, _>>()?;
    let symbols = modules
        .iter()
        .map(|module| {
            LegacyModuleSymbols::bind(
                module,
                LegacyDeveloperMode::Disabled,
                SymbolBudget::default(),
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let definitions: Vec<_> = INPUTS
        .iter()
        .zip(&symbols)
        .map(|((name, _), symbols)| ScriptModuleDefinition::new(name, symbols))
        .collect();
    Ok(compilation::compile_bound_primitive_package(
        &definitions,
        ScriptPackageBudget::default(),
    )?)
}

fn main() -> Result<(), Failure> {
    let package = compile()?;
    let entry = package.function_for_declaration(0, 0).ok_or("No entry")?;
    let mut strings = StringHeap::new(StringHeapBudget {
        max_live_strings: 5,
        max_slots: 5,
        max_payload_bytes: 4096,
        ..StringHeapBudget::default()
    })?;
    let mut objects = ObjectHeap::new(
        &strings,
        ObjectHeapBudget {
            max_live_objects: 3,
            max_object_slots: 3,
            ..ObjectHeapBudget::default()
        },
    )?;
    let mut context = KernelHeapContext::new(&mut strings, &[], HeapWorkBudget::default())
        .with_objects(&mut objects, &[]);
    let result = package.program().execute_with_heap(
        entry,
        &[],
        KernelCallBudget::default(),
        &mut context,
    )?;
    if result.value != KernelValue::Number(LegacyNumber::Integer(1001)) {
        return Err("Package result differs".into());
    }
    let mut output = io::stdout().lock();
    writeln!(
        output,
        "result_integer=1001 modules={} functions={} compiler_steps={} peak_compiler_metadata_bytes={} runtime_steps={} heap_steps={} string_collections={} object_collections={}",
        package.usage().modules,
        package.functions().len(),
        package.usage().compilation.compiler_steps,
        package.usage().compilation.peak_metadata_bytes,
        result.usage.steps,
        context.usage().steps,
        context.usage().collections,
        context.usage().object_collections,
    )?;
    output.flush()?;
    Ok(())
}
