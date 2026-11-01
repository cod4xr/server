// SPDX-License-Identifier: AGPL-3.0-or-later

/// Program/execution resource, independent of native script capacity constants.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KernelResource {
    /// Live immutable string values.
    StringValues,
    /// Live immutable string payload bytes.
    StringPayloadBytes,
    /// Retained string slots, including vacant/retired slots.
    StringSlots,
    /// Logical retained string slot records.
    StringMetadataBytes,
    /// One complete string payload.
    SingleStringBytes,
    /// Shared heap operation/root discovery work across the call tree.
    HeapWorkSteps,
    /// Simultaneous logical literal cache, active indices, root IDs and mark flags.
    HeapScratchBytes,
    /// Complete verified module function inventory.
    Functions,
    /// Simultaneously live direct-call frames.
    Frames,
    /// Logical live direct-call frame-table payload.
    ExecutionFrameBytes,
    /// Conservative frame setup/argument-copy/reservation work.
    CallSetupWork,
    /// Input code instructions.
    Instructions,
    /// Verification scans/visits/edges.
    VerificationSteps,
    /// Logical code plus temporary state/worklist slots.
    VerificationMetadataBytes,
    /// Maximum reachable value stack depth.
    StackValues,
    /// Executed instructions for this invocation.
    ExecutionSteps,
    /// Logical reserved runtime stack bytes.
    ExecutionStackBytes,
    /// Complete local count, including parameters/uninitialized slots.
    LocalValues,
    /// Logical runtime initialized-state/value slots.
    ExecutionLocalBytes,
}
