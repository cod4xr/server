# ADR 0080: Closed-package include aliases

Date: 2026-11-01

Status: implemented; required Windows verification recorded below. Full GSC/S5 remains open.

## Decision

Extend the explicit supplied module definition with an opt-in borrowed include
directive slice. Its default remains empty. Resolve all directives against the
closed canonical catalog, with no source-derived I/O or automatic dependency
loading. Inventory active direct definitions first, then materialize one-hop
import aliases. Aliases retain their original global function index, visibility,
body-local scope and source maps, and never become further exports.

Keep imported origins distinct from direct definitions. Qualified lookup must not
expose imported names. Direct local definitions precede builtins; imported script
calls require the deferred-linking precedence assessed in 0079. A local function
reference can select a permitted imported target. Ordinary callers cannot reference
developer-only targets. Reject canonical duplicate includes, unknown files and
destination name collisions before publishing any package.

Add separately configurable aggregate include-edge and imported-target quotas,
checked wide arithmetic and complete alias/staging metadata. Reuse metered
canonical lookup with a caller-owned function or package ledger. Admit every edge,
binding/name visit, namespace comparison, hash entry and overlapping temporary
record payload before allocation. Preserve existing no-include package behavior.

Named calls choose direct local definitions, then supported builtins, then imported
aliases. Function references choose a permitted script definition/alias without
builtin fallback. This distinction follows the guarded archival body-emission and
later dependency-linking paths; it is an inference from those paths, not an executed
native compiler trace. Qualified lookups select only direct exports. An imported
developer function named like a builtin must not hide an otherwise permitted named
builtin call. A direct developer definition still enforces the normal visibility rule.

Canonical include resolution first counts possible aliases from each target's
active direct definitions. A per-source checked set rejects repeated target indices.
Complete edge rows and concurrent duplicate-check scratch are admitted before
fallible allocation. Alias records extend compiler metadata before inventory
construction, and the empty name table reserves direct plus imported capacity at
once. This avoids uncharged rehashing of all direct names during later imports.
Each target binding visit and complete name hash/equality work shares the compiler
ledger. Canonical lookup accepts either the function identifier ledger or the
package ledger, without allocating query names or adding a second resolver.

The final package reports include-edge/imported-alias counts separately from
executable functions. Alias/scope/staging memory drops after lowering; the shared
verified image retains original function/declaration/source ranges. Duplicate/
missing/collision failures carry the source include's numeric module/span, while
body errors retain original local declaration locations. No partial image escapes.

The package CLI supplies parsed include lexemes through with_includes and permits
only closed-catalog resolution. Single-file execution still rejects includes;
animation-tree directives remain rejected in both modes. Includes in unused
supplied files are still validated. Source phases and compiler work remain globally
admitted; neither path builds a host filename from an include spelling.

Defaults admit one million include edges and one million imported aliases, each
independent of direct function quotas and existing metadata/work admission. These
are configurable resource policies, not legacy precache/string/short-index caps.
Logical accounting excludes hash controls/spare capacity/allocator rounding and
RSS, as existing compiler scope accounting does. No new crate, dependency, VM
instruction or runtime host authority is introduced.

## Alternatives and consequences

Flattening imported aliases into direct definitions would change qualified lookup
and re-export behavior. Recursively copying dependency scopes would duplicate
transitive names and make cycles expensive or order-dependent. One-hop aliases
over the direct inventory preserve each body's original namespace without
topological sorting or host recursion. Simply searching imports before builtins
would change named calls; always searching builtins would break script references.
The origin-aware resolver preserves those distinct paths.

Caller-owned symbols still do not carry directives. Existing new(name, symbols)
calls keep their empty-directive contract; source-loading callers must reject
directives or explicitly supply the validated slice. The CLI does the latter.
Physical loading/mount/preprocessing contracts remain separate from name linking.

## Validation

Thirteen core integration tests cover source ownership, direct/indirect imports, one-hop exports, qualified misses,
builtin precedence, ordinary/developer rules, collisions including unused bodies,
missing/duplicate dependencies, forward/self/cyclic graphs, wide aliases/counts,
original function/file maps, shared runtime/heap admission and exact/every-shorter
resource failures. This includes 70,000 imported functions, separately 70,000
include edges across 70,001 modules, and 70,000-byte function names with exact
legacy long-name case identity. Every insufficient compiler-work prefix rejects;
exact/one-less edge/alias/work/metadata scopes stay checked. Heap pressure preserves
ordinary structs and plain/localized literals through imported function references.

Four real-process tests exercise nested original-file import scopes, builtin/
reference/developer distinctions, duplicate/conflict/non-export failures, numeric
alias metrics and original callee error locations. Existing unused-file directive
tests now distinguish unresolved includes from unsupported animation directives;
single-file directive rejection remains checked.

The include source guard now rejects 77 mutations, including selected body-emission
and dependency-linking control-flow roles. Its standalone check and integrated
syntax/99 structural replays pass. These constrain selected roles, not full native
compiler/loader/cleanup execution or error-order parity across dependency graphs.

Run the three-file example from the repository root in PowerShell:

```powershell
cargo run --locked -- --execute-gsc-package demo/root `
  --gsc-module demo/root examples/gsc/includes/root.gsc `
  --gsc-module demo/middle examples/gsc/includes/middle.gsc `
  --gsc-module demo/library examples/gsc/includes/library.gsc
```

Observed Windows debug result: integer 42, three executable functions, two include
edges/two imported aliases, 580 compiler steps, 3040 peak logical compiler metadata
bytes, 13 runtime steps and three peak frames. This is a finite behavior observation,
not a performance/RSS comparison. Windows workspace formatting, all-target Clippy,
debug/release tests (301 script frontend/229 script legacy/13 private script tests,
two script doctests and the server process suites), rustdoc with warnings denied,
and repository policy pass. Policy covers 904 sources/eight crates; the largest
source remains 287 lines. Linux/hosted checks remain unrun locally.

Mount/source precedence, preprocessing, automatic loading, source arrays/scheduling,
general host libraries and stock/mod acceptance remain separate open work.
