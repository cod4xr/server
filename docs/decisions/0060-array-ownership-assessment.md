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
reference functions at O0/O2 and initially recorded 262 identical traces. Native packed type
declarations compile on the host ABI, without asserting a native pointer/ABI layout.
Pool allocation, collision-free lookup, reference services and key output are
explicit bounded test models. Invalid targets run in separate processes whose
error callback verifies unchanged local/allocation state and exits successfully;
no error unwinder is modeled. Entity/read-only host branches abort if reached.
Stack-tag assertions, removal, native interning/allocation/GC and compiler/VM/game
execution are outside this corpus. The selected source-role guard rejects 69
scoped replacements/reorderings/constant mutations, including read-only branches;
guarded source is not execution evidence for those branches.

The corpus now records 306 traces after adding actual clear/set/remove/evaluate
and no-collision unlink/free helpers. It covers integer/string
head/middle/tail/missing keys, shared/unique arrays, string-reference callbacks,
temporary key creation and initialized Undefined parents. All 262 older rows remain
an unchanged ordered subsequence. The new compiler/VM/grammar/write-helper source
guard rejects 141 mutations in addition to the existing 69 copy/key-role mutations.

Source-derived write rules require care. General assignment evaluates RHS before
the lvalue; indexed lvalues evaluate index before parent reference setup. Literal
Undefined uses ClearArray directly, whereas the general VM array-index setter also
removes a computed Undefined after constructing its lvalue. Thus a missing-key
computed delete can temporarily create an entry, and an initialized Undefined
parent can autovivify to an empty array; direct clear rejects that parent. Grouped
primaries retain a separate syntax tag from direct Undefined. The general raw C
setter can retain an Undefined entry, but it must not be mistaken for the complete
VM array-assignment path. Invalid clear keys reject after shared-array detachment;
helper callbacks record that state without resuming or unwinding a GSC thread.

These compiler/VM branches are guarded source, not executed native routines. The
driver explicitly models their helper call sequences. Allocation, hash lookup,
reference and output services remain modeled. Native free-list publication is
checked across three deletions and reinsertion, while allocation stays monotonic;
collision relocation and allocator reuse are not exercised. Child deletion cases
use Integer/Undefined values, not object/vector destruction. Entity/read-only game
branches and whole-source execution stay open.

## Storage alternatives and requirements

The separate cycle-alias corpus now executes 20 O0/O2 traces using actual native
AddRefToObject, selected nonfreeing RemoveRefToObject decrements, Scr_EvalArrayRef,
CopyArray and key enumeration. Its modeled root/field ownership setup makes one
through four self-cyclic structs retain references to a live array. Dropping each
struct root decrements to zero excess references without clearing/freeing fields;
the outgoing array aliases remain. Detaching the live slot copies and reverses
its keys; a second write to the new unique slot does not allocate/copy again.
Clear/free/entity paths abort if reached. This is not native cycle detection,
tracing GC, allocator/freeing, unwinding or VM evidence. Existing 306 array traces
remain unchanged; a separate ref-role guard rejects 14 mutations.

This makes a collector compatibility problem concrete: reclaiming an unreachable
reference cycle can remove a native-retained alias and suppress an observable COW
copy/key reversal. Simply counting reachable aliases is insufficient. A possible
bounded solution is to identify garbage retained by cycles through indegree
peeling, then preserve a logical alias marker on live arrays reached by that
garbage before reclaiming payload. The marker must stay on the old array, not its
fresh COW copies. This is an assessment candidate, not implemented/accepted
runtime architecture. It needs independent graph-model, resource/workload and
native-trace validation before array source activation.

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

Windows verification passed repository policy (669 sources/eight crates), rustfmt,
warnings-denied all-target Clippy and rustdoc, and full debug/release workspace
tests. The 306 native helper traces, 69 copy/key mutations, 141 write-role mutations
and all 99 existing structural replays
regenerate. Linux/hosted CI and native game-client execution were not run.
