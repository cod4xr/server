# Assessment 0092: Native switch scopes and fallthrough cleanup

Date: 2026-11-01

Status: selected Windows evidence accepted; Rust scope lowering/source writes
remain open under ADR 0088.

Nine new cases execute the actual pinned switch planner, EmitSwitchStatement,
CompareCaseInfo and selected local/branch/loop/transfer helpers. They cover no
default, disjoint/common default paths, fallthrough, an empty case, an existing
outer local, nested if/else, an extra private local and continue through a switch
inside a loop. Complete before/after parent and allocated-block state, request
order and condition counts match independent expectations at driver O0/O2.

## Boundary

The modeled statement dispatcher selects the actual switch helper only in an
explicit switch driver profile. Integer-key validity/encoding, canonical IDs,
bounded sorting, ASTs, condition/body dispatch, temporary storage and output are
modeled. The bounded sort calls the actual native comparator. String cases and
unsupported diagnostics fail instead of supplying invented outcomes.

The shared AST helper now builds bounded ordered lists, with pair construction
using that same leaf. Extraction recognizes a following section boundary for
the final native comparator function as well as following function headers.
All 34 corrected planning/36 branch/10 loop fixtures remain byte-identical.
There is no Rust runtime, crate, dependency or Cargo target change.

These are opcode requests and selected compiler-state effects, not compatible
bytecode or native parser/VM/value freeing. Native opcode optimization, alignment,
allocator reuse, qsort, error unwinding, general string-case semantics and full
script compilation are not executed. Windows passes; Linux/hosted CI is unrun.

## Lifetime consequences

Without a default, a case-only local remains private. Disjoint case/default
locals also clean separately; common initialized names join the parent. An
already materialized outer local survives while an additional case local is
removed. Empty case labels and nested branches retain their own block metadata.

The fallthrough case demonstrates removal before entering the next case body,
without an explicit break between them. Cleaning only at the final switch exit
would retain a temporary alias too long. The nested-loop case preserves the loop
continue path while switch break exits the switch, including distinct initialized
bit positions and public declaration metadata. Do not derive cleanup from a
single generic intersection of initialized facts or from braces/breaks alone.

Six admission regressions reject missing/duplicate cases, incorrect default joins,
reordered fallthrough cleanup, removal of common/outer cells and damaged nested
continue/condition/initialization state. Together the four scope suites have 25
admission tests. Existing local-removal and switch source-role mutations, 99
structural replays and repository policy remain required. Complete Rust gates
remain ADR 0088 because this assessment changes no Rust behavior.

Next, establish constant-condition scope profiles and implement an iterative,
budgeted Rust plan for supported physical local lifetimes, separate from current
definite-initialization facts. Developer statements are currently rejected by
source lowering and must stay rejected until their own cleanup is implemented.
Source indexed writes, general nested ownership, scheduling, full GSC and joins
remain open.
