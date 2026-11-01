// SPDX-License-Identifier: AGPL-3.0-or-later
use cod4x_script::execution::{KernelCallBudget, KernelHeapContext};
use cod4x_script::heap::{
    HeapWorkBudget, ObjectHeap, ObjectHeapBudget, StringHeap, StringHeapBudget,
};

use crate::app_error::AppError;
use crate::script_image::ScriptImage;
use crate::script_result::{self, ScriptResult};

pub(crate) fn run(image: ScriptImage<'_, '_>, entry: u64) -> Result<(), AppError> {
    let mut heap = StringHeap::new(StringHeapBudget::default()).map_err(|error| {
        tracing::error!(event = "script.failed", phase = "heap", kind = ?error.kind, "script heap setup failed");
        AppError::ScriptHeap(error)
    })?;
    let mut objects = ObjectHeap::new(&heap, ObjectHeapBudget::default()).map_err(|error| {
        tracing::error!(event = "script.failed", phase = "objects", kind = ?error.kind, "script object setup failed");
        AppError::ScriptObjects(error)
    })?;
    let mut context = KernelHeapContext::new(&mut heap, &[], HeapWorkBudget::default())
        .with_objects(&mut objects, &[]);
    let outcome = image.program().execute_with_heap(entry, &[], KernelCallBudget::default(), &mut context).map_err(|error| {
        let location = error.function.zip(error.cause.instruction).and_then(|(function, instruction)| image.location(function, instruction));
        let heap_usage = context.usage();
        tracing::error!(event = "script.failed", phase = "execute", kind = "runtime", cause = ?error.cause.kind,
            function = error.function, instruction = error.cause.instruction,
            module = location.as_ref().and_then(|value| value.module),
            offset = location.as_ref().map(|value| value.position.offset),
            line = location.as_ref().map(|value| value.position.line), byte_column = location.as_ref().map(|value| value.position.byte_column),
            steps = error.cause.work_steps, setup_work = error.setup_work, peak_frames = error.peak_frames,
            heap_steps = heap_usage.steps, collections = heap_usage.collections,
            object_collections = heap_usage.object_collections,
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
    let objects = context.object_heap_usage();
    tracing::info!(
        event = "script.executed",
        profile = image.profile(),
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
        object_collections = heap_usage.object_collections,
        heap_scratch_bytes = heap_usage.peak_scratch_bytes,
        live_strings = strings.live_strings,
        string_payload_bytes = strings.payload_bytes,
        string_slots = strings.slots,
        string_metadata_bytes = strings.metadata_bytes,
        live_objects = objects.map(|usage| usage.live_objects),
        live_fields = objects.map(|usage| usage.live_fields),
        object_key_bytes = objects.map(|usage| usage.key_bytes),
        object_slots = objects.map(|usage| usage.object_slots),
        object_field_slots = objects.map(|usage| usage.field_slots),
        object_index_buckets = objects.map(|usage| usage.index_buckets),
        object_metadata_bytes = objects.map(|usage| usage.metadata_bytes),
        "primitive GSC main completed"
    );
    Ok(())
}
