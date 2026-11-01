# ADR 0064: Object-aware reference transport and pressure roots

Date: 2026-11-01

Status: accepted and implemented for host reference transport.

## Scope

Extend the explicit borrowed KernelHeapContext with optional shared struct storage
and complete external object roots. Preserve the existing string-only API, budgets
and accounting. Validate the object store's associated string owner and all supplied
object references before allocating frames or resetting the invocation cache.
Opaque copied IDs still grant no ownership. Immutable executable object constants
remain forbidden, even when unreachable.

Permit checked host object arguments to travel through private locals, stacks,
direct/indirect calls and returns. During string pressure, gather both object and
string roots from every live frame, caller, external inventory and active literal
cache. First reclaim unreachable object graphs, then strings using every retained
object field plus supplied string roots. Both operations share the context's global
work/scratch budget, including simultaneous literal cache, temporary root arrays,
object mark/work queue and composite string root/mark records.

Each store operation preflights before mutation. An invocation or collection pair
is not transactional: a completed object collection can persist if the later string
collection fails. Track completed object and string collections separately. Preserve
all reachable objects, their nested strings and pending operands until publication.
Returned object IDs need explicit rooting before later invocations, like strings.

## Boundaries and acceptance

This adds reference transport/rooting, not source constructors/fields, array COW,
entity lifetimes, methods, scheduling or general object operators. Keep unavailable
operations explicit. Do not infer native refcount/GC/ABI parity from checked Rust
lifetimes. Object-aware source argument tests must exercise actual compiled calls;
isolated scalar source profiles remain unchanged.

Require deep iterative calls, caller/temporary/external/cache-only roots, nested
plain/localized strings, unreachable cycles, exact/one-less combined work/scratch,
failure after object collection, stale/foreign/wrong-bound stores, result transfer,
context reuse and string-only regressions. Full workspace checks remain mandatory.

Seven typed integration tests and one bound-source test now pass. They cover
standalone/direct/indirect transfer, invalid entry/external inventories, 12,000
iterative frames, caller-stack-only objects, cached-only strings, nested plain/
localized values and external object graphs. Every insufficient heap-work prefix
and one-less simultaneous scratch limit rejects while preserving reachable data.
Completed object/string collections are counted separately when a later phase fails;
late stale stored strings fail without a partial string sweep. A returned cyclic
graph survives explicit host roots and is reclaimed when omitted on context reuse.
Actual compiled GSC runs 1,000 loop/call concatenations while returning its supplied
object argument and preserving the nested localized string. These are Rust compiler/
execution ownership tests, not native object operator, refcount or collector parity.

Full Windows debug/release workspace tests, warnings-denied Clippy/rustdoc, rustfmt,
repository policy (709 source files/eight crates, largest 287 lines) and diff checks
pass. The script suites contain 244 frontend/163 legacy tests, six private units
and two doctests. Linux/hosted CI, native object operators, full mods and joins were
not validated. No dependencies, alternate targets or deployments were introduced.

The former execution/strings folder is now execution/heap because it coordinates
both stores. Accounting, validation, root discovery and object/string collection
remain separate leaf modules. No application or game-specific ownership enters
the script crate; pure/string-only callers retain their prior explicit rejection.
