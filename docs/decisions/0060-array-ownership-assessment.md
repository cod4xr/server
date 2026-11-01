# ADR 0060: Array ownership and observable copy semantics

Date: 2026-11-01

Status: native semantic evidence accepted; Rust storage/VM strategy still under assessment.

## Problem

Arrays cannot use ordinary mutable shared-object semantics. The pinned native
Scr_EvalArrayRef detaches a shared array before modifying the referencing local.
CopyArray recursively copies nested arrays but retains other object references.
These rules affect value identity, key iteration, memory, work and lvalue lowering.
The string-only tracing store does not establish these semantics for objects.

## Evidence and boundary

GetNewVariableIndexInternal2 prepends new keys; replacing an existing key preserves
its position. CopyArray traverses old siblings forward and inserts into the new
array using that path. Consequently each copy reverses key order, including nested
arrays. Scr_AddArrayKeys exposes the traversal. A second copy reverses it again.
This is now actual helper execution under modeled storage, not solely an inference
from source. Local-slot Scr_EvalArrayRef tests exercise autovivification, unique
writes and detachment with one through three aliases, including nested values.
Struct references retain identity while nested arrays become independent.

IsValidArrayIndex accepts unsigned words below 0x800000: script integer indices
are 0..8,388,607, not a signed range. Negative words reject. Internal name decoding
also has string, pointer and signed integer domains; those internal representations
do not make pointers or negative integers valid script indices. GetArraySize counts
entries rather than the highest integer index plus one. Native size/ref fields
are 16-bit; encoded names are 24-bit. None is a Rust engine storage ceiling.

The generator runs extracted key/allocation/copy/sibling/key-enumeration/local-slot
reference functions at O0/O2 and records 262 identical traces. Native packed type
declarations compile on the host ABI, without asserting a native pointer/ABI layout.
Pool allocation, collision-free lookup, reference services and key output are
explicit bounded test models. Invalid targets run in separate processes whose
error callback verifies unchanged local/allocation state and exits successfully;
no error unwinder is modeled. Entity/read-only host branches abort if reached.
Stack-tag assertions, removal, native interning/allocation/GC and compiler/VM/game
execution are outside this corpus. The selected source-role guard rejects 69
scoped replacements/reorderings/constant mutations, including read-only branches;
guarded source is not execution evidence for those branches.

## Storage alternatives and requirements

Do not choose a collector from the string workload alone. A shared mutable array
handle would change legacy scripts. Unconditional deep cloning on every write
would make repeated unique insertion quadratic. Persistent maps need a concrete
solution for native copy order and nested-array copies, not just root snapshots.
Reference-counted standard containers require a fallible allocation strategy,
cycle policy and workload evidence. A generational arena with counted managed
value locations is a candidate, not an implemented or accepted VM architecture.

Any selected approach must provide checked wide store/slot/generation identities,
separate array and struct behavior, content-based plain-string keys, independent
count/payload/retained-metadata/work/scratch admission and atomic failed writes.
Retained hash storage must stay accounted after deletion. Copying raw Rust IDs
does not establish managed ownership; local/stack/call/host copies and transfers
need one explicit contract. Reclamation must trace nested keys/values, active
frames, caches, external handles and pending results, including cycles. Native
recursive copying/destruction must become admitted iterative work.

## Acceptance before activation

Prototype the selected strategy with 70,000 keys/objects, bounded churn, independent
aliases, deep nested arrays, shared structs/cycles and content-equal string keys.
Measure logical visits/retained storage and elapsed time; report RSS separately
only if measured. Require exact/one-less budgets, overflow/stale/foreign/generation
tests and atomic resource failures. Replay applicable native cases through Rust
and classify capacity expansion separately. Audit compiler/VM lvalue evaluation,
autovivification, deletion and error order before lowering source writes. Then
integrate complete roots, calls/returns and cancellation/failure cleanup before
allowing array instructions in published executable modules.

This assessment adds no Rust array/object execution and closes no complete legacy
GSC workstream. It does not alter existing string ownership or introduce a new
crate, dependency, native runtime linkage or serialized instruction ABI.

Windows verification passed repository policy (650 sources/eight crates), rustfmt,
warnings-denied all-target Clippy and rustdoc, and full debug/release workspace
tests. The native corpus/69 mutation checks and all 99 existing structural replays
regenerate. Linux/hosted CI and native game-client execution were not run.
