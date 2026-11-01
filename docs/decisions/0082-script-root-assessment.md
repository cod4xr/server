# Assessment 0082: Native script root bindings

Date: 2026-11-01

Status: selected source evidence and implementation constraints. No Rust root
instruction or source activation is implemented by this assessment.

## Evidence

The pinned revision is ddd65fce1a1093017bad2dbb6840d75d11c0d56f. The new
script_root_reference.py reads only fully hash-verified source snapshots and
rejects 146 scoped mutations. It is called by structural grammar checks and CI.
The guard checks selected C functions, opcode/type identities, primary/compiler
and VM dispatch slots, ordered operations, and three shared-tail fallthroughs.
It does not execute the native compiler, VM, ABI wrappers or lifetimes. Existing
grammar checks still reproduce 99 structural replays without changing fixtures.

Scr_VM_Init clears level/game/anim identities. Scr_InitSystem allocates level and
anim through AllocObject, alongside separate time/pause structures. In contrast,
Scr_AllocGameVariable lazily obtains a value cell with AllocValue and initializes
that cell with SetEmptyArray. Its nonzero guard prevents repeated initialization.
These are distinct namespaces/roles, not interchangeable ordinary struct IDs.

Native GetLevel/GetAnim push pointer-tagged values from global parent identities
and add an object reference. GetSelf obtains the current thread identity from
the frame, calls Scr_GetSelf and shares an AddRefToObject tail. Scr_GetSelf reads
the receiver stored in a live thread/child-thread parent. AllocThread and
AllocChildThread store the supplied receiver. Scr_ClearThread releases the old
receiver; Scr_StopThread clears first, replaces it with level and adds its ref.
Thus self must not be inferred from a local variable, function image, or a global
always-level default. Thread scheduling/entity semantics remain separate work.

GetGame reads the global value-cell ID and shares the Scr_EvalVariableExtern
tail, copying both value and tag. The guarded GNU wrapper and C fallback call
Scr_EvalVariable, whose selected C body reads the CHILD namespace and calls
AddRefToValue. GetGameRef selects the global value cell for writable reference
handling. A borrowed ArrayId data snapshot cannot replace that writable managed
location: future indexed writes must select the proper owning cell for COW.

GetLevelObject/GetAnimObject use receiver temporaries instead of pushing owned
values. GetSelfObject also checks IsFieldObject. A source implementation can
lower supported field operations through general checked value/field machinery,
but must preserve supported semantics and reject unsupported array/entity cases.
The archived 32-bit offsets, packed tags and opcode numbers are evidence only;
they are not new Rust capacities, serialized instruction formats or ABI promises.

## Next bounded implementation

Use a typed, explicit caller-owned root view for an invocation: ordinary shared
level/current-object/anim identities and an existing host-managed game-array value
location. Keep its binding IDs fixed for that invocation and validate all supplied
roots before entry cloning. Root objects must remain roots through every pressure
collection even when no current operand refers to them. Array locations already
root their data, but a game read must clone a new VM owner under ADR 0081 rather
than insert a raw borrowed host handle into an owning cell.

Missing root context must fail at the executed root operation; do not fabricate
objects in the offline runner. Invalid supplied root inventories must fail checked
admission. Keep pure/scalar use without supplied roots unchanged. Type categories
must remain explicit: the first profile can support ordinary object receivers,
not entities, animations, thread objects or a complete server world. An anim root
object alone supplies no animation assets or #animtree behavior.

Root operations belong inside script execution, without script depending on
server-core or importing ambient server state. Immutable programs must not embed
heap identities. Preserve complete source maps, owned package images, developer
visibility, private numeric diagnostics, global invocation work/fuel and overlapping
cache/registry/root scratch. Existing client/native limits must not become internal
slot widths. Add no dependency, crate, callback/plugin registry or generic host
framework for this bounded context.

This profile must not enable general source array creation/indexing/stores, nested
owned fields, implicit game replacement, native logical object RC/cycle markers,
method receiver changes, thread scheduling or game lifecycle persistence claims.
Proposal 0071 and ADR 0075 remain required for mixed/nested source ownership.

## Acceptance

Check all four root reads, aliases through locals/direct/indirect calls, fields on
supported ordinary roots, top-level result transfer and root-private Debug/logging.
Test missing/foreign/stale/wrong-tag bindings, exact/one-less budgets, every failed
work/instruction/setup prefix, caller/callee/pending cleanup, deep calls and wide
field names. Force paired pressure collection while root-only struct/string graphs
are otherwise absent from operands. Confirm game reads never consume the host
owner and discarded/failed reads release every VM copy before the next operation.

Keep the runner honest about absent world context with a real-process failure
test. Run the complete existing workspace gates and source/structural guards after
Rust implementation. The last complete Windows Rust gates are commit 097fb90:
310 frontend/235 legacy/15 private script tests/two doctests. This assessment's
Python/CI changes pass root mutations, structural checks and repository policy
(925 source files/eight crates/largest 287 lines); Linux/hosted CI is unrun.
No full GSC workstream or S5/join gate closes here.
