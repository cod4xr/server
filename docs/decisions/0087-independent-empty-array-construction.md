# ADR 0087: Independent empty-array construction

Date: 2026-11-01

Status: accepted for independent empty construction. General writes, stored/nested
arrays, native logical lifetimes and complete GSC remain open.

## Problem and bounded lifetime review

ADR 0086 admits reads only because nonempty executable arrays derive from live
host locations. A constructor creates another origin and requires a new review.
It is safe to admit independent empty data while keeping every way to populate
independent data closed: indexed writes, stored arrays, array-returning builtins,
host callbacks and game replacement remain unavailable. A fresh source array
therefore has no outgoing payload references throughout its invocation. Copies,
moves, calls and returns can use the existing explicit managed owner registry.
Nonempty host-origin arrays still obey 0085/0086's backing-root argument.

This review does not solve general source arrays. Allowing writes, persistent
array fields, nested arrays, scheduler captures or another populated origin needs
the appropriate subsequent ownership review, including 0071/0075's ordinary
logical lifetimes and retained-cycle aliases where those edges become possible.

## Native evidence and implementation

The pinned parser's empty primary is rule 56/tag 0x42. The selected compiler
branch emits OP_EmptyArray (0x25) with one stack result and a source position; it
does not manufacture a compile-time array pointer. The selected VM branch pushes
a pointer-tagged cell and calls Scr_AllocArray. The C allocator obtains a parent,
marks it external/array, sets excess refs and the count-overlaid entity word to
zero, and restores diagnostic state. Existing native array corpora execute this
actual allocator under modeled bounded pool services. The new scoped role guard
also protects emission/type/dispatch/order. It does not execute the native compiler
or VM, establish allocator reuse/freeing parity, or license source writes.

Use a typed EmptyArray instruction, with a verified 0-to-1 stack effect and
explicit object context. Bound source emits it for []; restricted constant-only
source modes retain their existing boundaries. Never embed a runtime array ID in
immutable code. Prepare complete frame-owner storage and cleanup credit before
allocation. Cover a fresh location in the pending-owner slot before any fallible
accounting/probe/publication. Calls/returns then use the existing transport rules;
top-level return transfers one host owner requiring explicit release.

Array data/value allocation must share invocation work and scratch, preflight
complete quotas and remain atomic. On eligible object/storage pressure, perform
one paired collection/retry with complete current/caller/cache/world/managed roots.
No unconditional whole-heap scan on each construction. Exhausted failure cleanup
cannot allocate, borrow remaining work or discard host owners. Deferred reclamation
of unowned empty payloads is safe under this profile; no observable array identity
comparison, reference/cycle edge or data mutation becomes available.

## Validation

Test standalone and source/package construction, returned ownership, independent
data, copy/move/callee exit, empty size/missing reads, precise mapped context/type
errors, immutable-constant rejection and retained write/nesting/game gates.
Exercise exact/one-less frame/storage/work/scratch, every failed heap/instruction/
setup/publication prefix, pressure retries and same-context recovery. Use wide
simultaneously live values and bounded construction/discard churn to expose stale
owner history or unrooted results. Include caller/root/string graphs and combined
registry/cache/root/mark scratch. Run native array/empty-role/structural/suite
guards, runnable source/CLI examples and complete Windows gates before acceptance.
No new crate/dependency or generic host factory is needed. Full GSC/S5 and joins
remain open.

Eleven new source tests, two standalone tests and three process tests pass. They
cover independent returned data/host release, copied owners, source/package calls,
restricted compiler profiles, missing-context recovery, forbidden stored edges,
every shorter instruction/setup/heap/publication prefix, owner-table growth across
37 held arrays and twelve exact/one-less storage/frame/scratch resources. A paired
pressure test preserves caller results, two cached keys, host graphs and an unused
world-only struct/string graph through every shorter heap prefix. Existing typed
immutable-array-constant rejection remains unchanged.

Wide checks create 70,000 independent source data nodes; 100,000 constructions
reuse one managed value slot/two data slots under 4 KiB scratch with 49,999 paired
collections. A first allocation at frame 12,001 uses one data/value slot and
releases its sole owner when the last deep return lacks one work unit. These are
correctness/resource checks, not comparative performance claims.

Run `cargo run -p cod4x-server --locked -- --execute-gsc examples/gsc/empty_arrays.gsc`
for integer 1. Process tests cover opaque array output, numeric logical storage
metrics, private located index errors with zero remaining VM owners, and readable
logs explicitly selected with --human-logs. Development defaults stay pretty;
release defaults stay JSON. Success reports array data/value/slot/metadata counts;
runtime failures report data/value/slot counts after prepaid cleanup. Unowned
empty data may remain until collection; counters do not claim process RSS.

Full Windows formatting, all-target denied-warning Clippy, debug/release workspace
tests, denied-warning rustdoc, policy and diff checks pass. Both profiles execute
345 frontend/242 legacy tests plus fifteen private script tests/two doctests and
the three new process tests. The former empty-constructor rejection test is
intentionally replaced by the retained indexed-write/game-replacement gate;
all other prior test names remain. Native 306 array/20 alias/210 lifetime/100 read
O0/O2 traces, 39 constructor mutations, 99 structural replays and twelve suite
policy regressions pass. Policy checks 968 sources/eight crates/largest 287 lines.
No Cargo target directory/crate/dependency was added. Linux/hosted CI is unrun.
