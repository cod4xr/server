# ADR 0055: Explicit heap-backed primitive execution

Date: 2026-10-31

Status: accepted for typed execution and supplied source arguments; source literals remain open.

## Decision and alternatives

Connect the existing string store to typed execution through an explicit borrowed
heap context. Copy opaque checked StringId values through stacks/locals/arguments/
returns, without copying payload or adding reference counting. Immutable verified
code rejects runtime StringId constants, including unreachable ones: otherwise
unexecuted constants would silently become hidden roots tied to a mutable store.
Source literal tables/cache admission remain a separate subsequent change.

An implicit temporary heap would produce dangling results. Per-call budgets would
permit recursive work amplification, and append-only runtime allocation would
retain overwritten loop temporaries. Use one caller-owned context/work budget for
the entire invocation tree. Existing pure entry points reject string arguments.
The explicit entry points use legacy argument-prefix binding and preserve existing
fuel/setup/stack/local/frames admission and located sanitized errors.

Implement plain-string addition and equality/inequality by resolved byte contents.
Do not invent numeric formatting, localization, lexicographic ordering or string
index semantics. Binary host bytes are owned data, not native NUL/interner parity.
Derived KernelValue equality remains data equality, not language comparison.

## Reclamation and failure ownership

Attempt allocation first. On reclaimable live/payload/slot/metadata pressure, scan
complete current/caller stack/local slots and supplied external roots, reclaim once
and retry. Operands stay on the stack until allocation finishes. No collection runs
between popping a return value and publishing it to the caller/result. Discovery,
temporary root IDs, mark flags, failed attempts and retry work share context budgets;
uninitialized/nonstring slots still cost discovery visits. Duplicate roots are valid.
Single-value/work/overflow/type failures do not trigger collection. Store mutation
on an invocation failure is possible; individual storage operations remain atomic.

Caller-owned copied IDs are not ownership leases. Callers must supply every handle
that must survive beyond VM locals as an external root. Arguments initialize private
locals but do not implicitly root unrelated host copies after overwrites. Returned
IDs remain in the borrowed caller-owned heap; future invocations must root them.
Context inspection provides sanitized aggregate work/collection/scratch counters
after both success and failure. Heap payload/results never enter operational logs.

## Validation and open boundaries

Require byte equality versus reference identity, addition, copied/returned data,
foreign/stale arguments, rejection of runtime constants even when unreachable,
caller/local/operand/external roots under pressure, bounded churn, failed collection,
work sharing across calls, fuel/error paths, wide/large values and existing scalar
fixtures. Prove exact and one-less work/scratch admission. Source literals and CLI
strings, mutable arrays/objects/cycles, scheduling, host/game roots and full native
cleanup/VM/mod/platform acceptance remain open. This defines no serialized ABI.

## Implemented evidence

Fifteen kernel tests cover ownership, operators, runtime constant rejection,
isolated caller-stack/callee-operand/external roots, a 12,000-frame collection,
exact/one-less work/scratch, each retained pressure resource, noncollectable limits,
global work across calls/reused contexts, fuel failure and 4,096 hostile inventories.
A 100,000-iteration VM workload uses four slots/six live payload bytes, performs
99,998 collections and executes 1,200,008 instructions. This is bounded-lifetime
evidence, not a comparative timing/RSS benchmark. The native plain-concatenation
corpus passes literal decode/host insertion/VM call/return: 54 helper successes
match and 54 capacity failures intentionally expand. Native interning/cleanup is
not executed; equality by byte contents is source grounded, not a new native oracle.

Two real bound-source tests exercise supplied strings through locals/for/calls under
pressure after source ownership ends, and preserve the failing addition token's
source map without exposing names/payload. Source literal construction, includes,
localized/mixed coercion, string indexing and CLI string output remain unsupported.
All prior primitive entry points still reject runtime string arguments without
an explicit context; no hidden heap or serialized/native ABI is introduced.
