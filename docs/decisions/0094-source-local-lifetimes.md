# ADR 0094: Metered source local lifetimes

Date: 2026-11-01

Status: accepted for the bounded source lifetime component on Windows. Source indexed writes remain
rejected under ADR 0088; full GSC and joinable gameplay remain open.

## Behavior

Bound modules and closed packages now plan declaration tables separately from
initialized-value facts and emit explicit Undefined stores to end private local
lifetimes. Ordinary braces share their parent scope. One-sided/disjoint branches
clean private aliases, common reaching branches preserve public cells, and an
already-created outer binding survives a branch. Later direct declarations do not
retroactively keep an earlier branch alias alive.

Loop continuation declarations use native union rules. Constant-true loop breaks
add common exit names; dynamic-condition breaks do not. For initialization, body,
update and continue destinations remain distinct. Public prefixes expand during
emission-time inheritance: a fixed maximum of two old counts loses displaced
names. Switch fallthrough cleans before the following label's target, and nested
switch break preserves the outer loop's continue destination. Return and error
paths use the existing prepaid frame-owner cleanup.

The planner, condition classifier and emission tasks are iterative. Blocks,
names/positions, control/label rows, context lists, pending tasks and decoded
constant payloads pay logical metadata/control admission before allocation.
Visits, name movement, intersections/unions, decoding and concatenation pay work.
Auxiliary tasks share the existing task limit without charging a second unrelated
task representation. Local IDs/counts remain wide. Logical budgets exclude spare
capacity, hash control bytes, allocator rounding, fixed fields and process RSS.

Initialization flags move to a separately admitted indexed vector. This makes
clearing one removed binding constant-time and prevents a cleared partial-break
cell from becoming readable at a later switch join. Snapshots remain flat and
metered. Planning a name, creating an Undefined cell and initializing its value
are distinct events.

## Constant and source boundaries

The source classifier composes the established numeric, integer, vector,
Undefined and same-kind string kernels for supported primary/binary expressions.
Groups are iterative; vector components retain right-to-left evaluation.
Prefix/logical root forms follow the native planning dispatch's dynamic path.
Calls, roots and references remain dynamic. Unsupported type operations remain
runtime errors before entering the body; their classification does not claim
native compile-error or unwinding parity. Existing source initialization is still
conservative about zero-iteration loops. This is not the complete native compiler.

Standalone pure/primitive source profiles do not acquire mutable heap authority
or this bound-module planning profile. Developer statements, indexed writes,
nested/stored Array values, root/game writes, scheduler ownership and general
ordinary logical RC/cycle behavior remain gated. Existing initialized-path checks
are a Rust source policy; native helper state does not prove all compiler acceptance
or diagnostics, including conservative nested-break initialized bitmaps.

## Additional native evidence

The integer-switch driver adds two scenarios after its original nine, preserving
all original scenario bytes and the other four fixture families. Both execute the
actual selected pinned helpers at driver O0/O2. A partial nested break removes its
private leaf before jumping. Both-aborting nested arms leave the case root's
initialized bitmap unchanged even though common names appear in the declaration
table. The selected native emitter also requests an unreachable remove count 255
after those arms. That byte underflow stays explicit in the oracle; Rust emits no
cleanup on an unreachable path. It is not a requested live removal of 255 owners.
The harness still models dispatch/output/allocation services and executes no native
parser, full bytecode, VM freeing or error unwinding. Three new admission tests
bring the five scope admission suites to 35 tests.

## Acceptance

Twenty-two new scope integration tests and one caller/cache/world pressure test
cover branch/loop/switch alias quotas, nested partial breaks, inherited public
prefixes, constant value rules and all failed compilation/instruction/setup/heap
work prefixes. Exact and one-less compiler/runtime resources are checked. Failed
and successful calls share a context repeatedly without retained VM owners.
Private removal crosses 70,000 bindings and 12,000 nested bound branches; a
separate 12,000-deep alternating while/for test releases planner contexts iteratively.
100,000 private alias iterations use 4 KiB scratch beside 70,001 unrelated host
owners; a separate 100,000-constructor pressure test uses two data/value slots
and performs 49,999 paired collections. Collection work is bounded but can scale
with the retained owner arena; no constant-cost collection claim follows.

The local_scopes source example returns integer 42. A real closed-package process
test uses it through an include and checks readable development logs and zero
live VM owners. Existing script test inventories must remain intact. Required
gates are complete Windows formatting, all-target denied-warning Clippy,
debug/release workspace tests, denied-warning rustdoc, five native scope generators,
98 local/39 switch mutations, 99 structural replays, 35 scope admission tests,
twelve suite-policy regressions, repository policy and diff checks.
Linux and hosted CI remain unrun. No new dependency, crate or target directory.

All required Windows gates pass. Each complete workspace run executes 367
frontend/257 legacy tests, fifteen private script tests and two script doctests;
the final additional deep-loop test passes separately in both profiles, bringing
the validated frontend inventory to 368. Every previous 345 frontend/257 legacy
test name is preserved. Repository policy checks 1042 sources/eight crates, with
the largest source still 287 lines. All five native generators and 35 admission,
98/39 mutation, 99 structural and twelve suite-policy regressions pass. The
existing 306 selected array traces and 141/61 write/lifetime mutations also pass.

Next, review source indexed assignment against these lifetime boundaries and the
accepted typed writable-cell instructions, including observable COW/key order,
literal versus computed Undefined and independently populated result graphs.
