# ADR 0077: Explicit bound script packages

Date: 2026-11-01

Status: implemented; Windows verification recorded below. Full GSC/S5 remains open.

## Scope

Compile a caller-supplied catalog of canonical virtual module names and already
validated LegacyModuleSymbols into one immutable verified image. Local names
remain local; file-qualified calls and function references resolve through the
closed supplied catalog. All active definitions are inventoried before lowering,
allowing forward/cyclic call graphs. Retain disabled declaration metadata and
ordinary/developer visibility rules. A qualified missing function must not fall
back to a builtin. No host I/O, include expansion, methods, threads, native plugin
authority, loader ordering or complete stock/mod compatibility is implied.

## Decision

Reuse existing statement lowering, staged code/maps, global module compiler ledger
and complete kernel proof. Name keys include a wide source-module index. Keep
caller-supplied module order stable for declaration/function ranges; use a separate
budgeted sorted index over owned canonical names for lookup. A fallible iterative
merge sort and prepaid binary comparisons avoid hidden allocations/probe scans
on module names. Every canonicalization/copy/compare/row/move needs shared work
admission and simultaneous metadata accounting before allocation.

Own module names, declaration/function ranges and instruction spans after source/
tree/symbol drop. Link all calls/references directly to wide function indices in
the shared verified image; existing module-identity checks protect indirect calls.
Locate failures by numeric source-module/declaration indices and original byte
spans, without logging names or paths. Publish no partial package on any failure.

Keep isolated/single-module APIs and their unsupported qualified-call behavior
distinct. Default package quotas are configurable resources, not native filename,
loaded-script, short-index, call-chain or fixed include limits. Caller-owned source/
CST/symbol storage and kernel proof remain separately admitted scopes.

The existing local function inventory now keys names by source-module index and
legacy identifier. Single-module compilation supplies index zero and no package
scope, preserving rejection of file-qualified calls there. Qualified misses never
fall back to builtins; local script definitions still precede builtin fallback.
No runtime opcode changes, new crate, third-party dependency or plugin is required.

Global compiler metadata admits namespace rows/payload, sorted index, overlapping
canonical borrowed views/sort scratch, local inventory and staged code/maps.
Namespace comparisons and moves share compiler work; qualified queries additionally
share per-function identifier admission. Kernel proof and caller source/CST/symbol
storage retain their documented separate scopes. Final usage exposes retained
names/module boundaries/maps separately from immutable kernel code and peak
compiler metadata. Logical accounting excludes allocator/spare capacity and RSS.

The public input deliberately accepts already-bound symbol tables. These tables
do not carry include/animation directives. The caller must reject or resolve those
directives in its source-loading phase; this API does not validate a filesystem
dependency closure or silently claim include processing. The runnable example uses
two explicit compile-time repository sources and grants no runtime file authority.

## Alternatives and consequences

Separate verified images would require new cross-image call/reference semantics
and coordinated runtime admission; one verified image reuses checked function
identity and existing complete invocation budgets. A hash table for canonical
module paths would hide comparison/probe cost; a sorted index permits explicit
prepaid comparisons without rearranging source modules. Retaining source/tree
lifetimes would simplify staging but prevent independent owned execution; only
canonical names and numeric maps survive compilation.

Existing single-module semantic APIs remain available. Their logical active-name
row accounting increases by the actual namespace-index field. New callers opt into
the package API and separately admit source phases. Package defaults (16,384 module
rows, 64 MiB raw catalog names) are configurable resources, not compatibility
ceilings or claims about safe production settings.

## Validation

Thirteen package integration tests execute direct/indirect qualified calls,
same-name isolation/shadowing, canonical collisions, forward/recursive graphs,
developer/inactive/empty modules, owned source-drop execution, private numeric
locations, strict unknown-file/function failures and complete namespace/lowering/
proof budget boundaries. Every insufficient compiler-work prefix rejects without
publishing a package; exact and one-less global resources remain checked.

One test compiles 66,000 source modules/functions in reversed catalog order and
calls the final wide target. Another executes 12,002 simultaneous cross-file frames
under explicitly increased setup-work admission, with exact/one-less fuel and
frame checks. A 70,000-byte module name survives caller-buffer destruction. Heap
pressure preserves caller/callee objects and both string tags across file calls.
These are finite component expansions, not native whole-compiler/VM/loader parity.

`script_package_reference.py` verifies selected pinned archival compiler blocks
for canonical file identity, local/per-file function namespaces, unknown-function
errors and local-only builtin fallback. It rejects 38 scoped removal/reordering
mutations and is wired into syntax checks and CI. It does not execute the native
compiler or validate its loader/include/deferred linking/cleanup behavior.

Run `cargo run -p cod4x-script --example script_package --locked` for the supplied
two-file GSC example. Observed Windows debug result: integer 1001, two modules/two
functions, 552 compiler steps, 4601 peak logical compiler metadata bytes, 22,022
runtime steps, 638,405 shared heap steps and 998 string/998 object collections.
This checks behavior under small heaps; it is not a performance or RSS comparison.

Windows workspace formatting, all-target Clippy, debug/release tests (including
288 frontend/229 legacy/13 private script tests and two script doctests), rustdoc
with warnings denied and repository policy pass. Policy covers 883 source files
and eight crates; the largest source remains 287 lines. Linux/hosted checks, loader/include support,
source arrays/ownership, self/methods/threads/scheduling, general host services and
recorded stock/mod execution remain open.
