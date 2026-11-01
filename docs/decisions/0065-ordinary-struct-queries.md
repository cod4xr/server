# ADR 0065: Ordinary struct definedness and size

Date: 2026-11-01

Status: accepted and implemented for checked ordinary structs.

## Decision and limits

Checked ordinary struct references already travel through the explicit object
context. Support their existing definedness and size behavior without inventing
entity/thread subtypes or confusing storage metadata with script semantics.
IsDefined returns true for a live ordinary struct, including an empty one. Extra
arguments retain the required-first/evaluated-extra policy. Size validates the
reference and returns integer one, independent of its field count. Charge metadata
validation against global heap work before publication; do not walk its fields.

All values still enter through an object-aware context and checked store identity.
Pure/string-only invocations and immutable object constants retain their rejection.
This does not add constructors, field access, arrays, general object operators,
entity/thread lifetimes, native reference cleanup or full mods. Supplied nonowning
object results retain the complete-root contract from ADR 0064.

## Evidence

The pinned header identifies ordinary structs as VAR_OBJECT=18, arrays as 21.
Scr_EvalSizeValue reads the parent namespace, returns the native u16 size word only
for arrays and integer one for other pointer subtypes, then removes a reference.
The existing selected size-role guard now additionally protects the parent-pool
offset, rejecting 46 mutations. The new generator executes the actual C helper
with actual selected packed declarations on the host ABI and bounded modeled
parent records/ref callbacks at O0/O2. All 180 outcomes match. Fifteen ordinary
struct cases replay through Rust; 165 other-subtype cases stay helper-only. The
input union size word is not a native struct field count. No native pool, cleanup,
unwinding, pointer ABI or whole-VM conformance is claimed.

The existing 351-row isolated archival definedness/modeled-admission corpus now
replays eight additional ordinary-struct rows: 64 Rust replays total, 56 results
and eight missing-argument errors, with 287 helper-only rows. This is not execution
of the original function/ABI or entity/thread lifetime services. The four actual
C type predicates additionally map their pointer tag to the available ordinary
struct: 224 of 832 rows replay (32 results/192 arity errors), with 608 helper-only
rows. Canonical fixture outcome rows remain unchanged.

Typed tests admit 70,000 struct fields while Size still uses three global heap
work steps including entry validation. Exact admission succeeds; one-less work
rejects at the original operator without mutation. Compiled GSC calls combine
size and definedness with evaluated extras. Source constructors/fields and native
object equality/casting still require separate evidence and implementation.

Full Windows workspace debug/release, warnings-denied Clippy/rustdoc, formatting,
repository policy (714 sources/eight crates/largest 287 lines) and diff checks pass.
The source-role and native pointer-size/string-access/definedness/type-predicate
generators regenerate. Current script suites contain 245 frontend/165 legacy
tests, six private units and two doctests. Linux/hosted CI, native lifetime services,
full mods and joins remain open. No dependencies or alternate targets were added.
