// SPDX-License-Identifier: AGPL-3.0-or-later
use cod4x_script::compilation::CompiledModule;
use cod4x_script::execution::{KernelCallBudget, KernelHeapContext};
use cod4x_script::heap::{HeapWorkBudget, StringHeap, StringHeapBudget};
use cod4x_script::source::SourceText;

use crate::app_error::AppError;
use crate::script_result::{self, ScriptResult};

pub(crate) fn run(
    source: &SourceText<'_>,
    compiled: &CompiledModule,
    entry: u64,
) -> Result<(), AppError> {
    let mut heap = StringHeap::new(StringHeapBudget::default()).map_err(|error| {
        tracing::error!(event = "script.failed", phase = "heap", kind = ?error.kind, "script heap setup failed");
        AppError::ScriptHeap(error)
    })?;
    let mut context = KernelHeapContext::new(&mut heap, &[], HeapWorkBudget::default());
    let outcome = compiled.program().execute_with_heap(entry, &[], KernelCallBudget::default(), &mut context).map_err(|error| {
        let span = error.function.zip(error.cause.instruction).and_then(|(function, instruction)| compiled.instruction_span(function, instruction));
        let location = span.as_ref().and_then(|span| source.location(span.start).ok());
        let heap_usage = context.usage();
        tracing::error!(event = "script.failed", phase = "execute", kind = "runtime", cause = ?error.cause.kind,
            function = error.function, instruction = error.cause.instruction, offset = span.map(|span| span.start),
            line = location.map(|value| value.line), byte_column = location.map(|value| value.byte_column),
            steps = error.cause.work_steps, setup_work = error.setup_work, peak_frames = error.peak_frames,
            heap_steps = heap_usage.steps, collections = heap_usage.collections,
            heap_scratch_bytes = heap_usage.peak_scratch_bytes, "primitive GSC execution failed");
        AppError::ScriptExecution(error)
    })?;
    let result = ScriptResult::resolve(outcome.value, &context).map_err(|error| {
        tracing::error!(event = "script.failed", phase = "result", kind = ?error, "script result reference failed");
        AppError::ScriptString(error)
    })?;
    result.write_stdout().map_err(|error| {
        tracing::error!(event = "script.failed", phase = "output", kind = ?error.kind(), "script result output failed");
        AppError::ScriptOutput(error)
    })?;
    let usage = outcome.usage;
    let heap_usage = context.usage();
    let strings = context.heap_usage();
    tracing::info!(
        event = "script.executed",
        profile = "bound_primitive",
        result_kind = script_result::kind(outcome.value),
        steps = usage.steps,
        setup_work = usage.setup_work,
        peak_frames = usage.peak_frames,
        peak_frame_bytes = usage.peak_frame_bytes,
        peak_stack_values = usage.peak_stack_values,
        peak_stack_bytes = usage.peak_stack_bytes,
        peak_local_values = usage.peak_local_values,
        peak_local_bytes = usage.peak_local_bytes,
        heap_steps = heap_usage.steps,
        collections = heap_usage.collections,
        heap_scratch_bytes = heap_usage.peak_scratch_bytes,
        live_strings = strings.live_strings,
        string_payload_bytes = strings.payload_bytes,
        string_slots = strings.slots,
        string_metadata_bytes = strings.metadata_bytes,
        "primitive GSC main completed"
    );
    Ok(())
}
