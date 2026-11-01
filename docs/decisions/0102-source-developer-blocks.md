# ADR 0102: Bound-source developer blocks and discarded validation

Date: 2026-11-01

Status: bounded source component accepted on Windows.

Bound modules and supplied packages propagate each symbol index's explicit
developer mode into lowering. Enabled /# ... #/ lists execute; disabled lists
validate source and then discard temporary code, literals, fields and switch
labels. Disabled top-level definitions retain their earlier inclusion policy and
remain excluded from compilation. Standalone function profiles still reject
developer statements. A list inside another list or a developer-only definition
reports NestedDeveloperBlock at its private source span.

Assessments 0099–0101 distinguish native developer-list emission, local-name
planning and discarded function names. The source compiler keeps those concerns
separate. Developer children receive their parent's public prefix without directly
publishing private names, initialized bits or abort state. The planner still
collects enclosing continuation/break evidence through developer children.
Enabled fallthrough clears private cells; returning/transferring paths use their
existing prepaid cleanup. Parent source validation resumes after the list, including
when an enabled return makes that retained continuation unreachable at runtime.
These are bounded source rules, not full native diagnostic/code-byte parity.

Developer-only function visibility applies inside active lists and restores on
exit. Discarded named calls/references validate identifiers and qualified path
shape/work without resolving a module or executable function. Argument expression,
initialization, literal, index, control and count checks still run. The existing
restricted immediate/receiver/thread feature gates remain explicit. Full builtin
probing and native argument/diagnostic ordering do not follow from the selected
EmitFunction evidence.

Unlinked validation uses a private pending marker with no kernel instruction or
target representation. Both emitting it outside discarded source and allowing it
to escape final resolution fail closed; two private tests guard those boundaries.
Discarded outer break/continue paths validate the legal target and local cleanup
without appending patches or initialization facts to retained outside constructs.
Nested loops/switches wholly inside a discarded region resolve locally before
their code is removed. No executable dummy function is added to the inventory.

One flat optional developer frame owns the admitted initialization snapshot and
code/table checkpoints. Iterative tasks handle regions; nested regions reject
without recursively owned frames. Discard work prepays flat row/instruction drops.
All work and peak metadata remain charged after truncation. Existing stable local
layout still includes planned private declarations, including discarded names;
this change does not implement local-slot compaction. Source bytes/CST, spare
vector capacity, fixed compiler fields and process RSS keep their existing scopes.

Function/module accounting now distinguishes retained instructions from
peak_instructions. Instruction admission includes the staged active prefix plus
the current temporary peak, even when developer code is discarded. Complete
compiler/global/function/proof overlap stays admitted. Compiler logs report both
counts without source values. Module record aggregation moves into its own small
file; statement queueing moves into the conditional helper to preserve function
size limits. These are internal Rust API/accounting changes, with no wire/native
bytecode ABI or new dependency/crate/test binary/target directory.

Seventeen source tests cover mode selection, discarded unknown/local/qualified
references and complete table removal, retained argument/initialization checks,
returns, visibility restoration, nested regions, break/continue/switch ownership,
private-alias COW order, exact and every-shorter compiler/runtime/setup/heap work,
independent one-less quotas and per-token encoded/decoded limits. Supplied include
packages preserve different per-module modes and shared temporary/staged admission.
Wide tests admit 70,000 regions and 12,000 interleaved nested branch contexts,
including flat budget failure. 100,000 private constructor iterations collect with
two data slots; 100,000 alias iterations fit 4 KiB scratch beside 70,001 host owners.

One negative stress profile records a separate collector limitation:
full object collection still audits all retained array-owner slots. Many host
aliases make repeated collection expensive even though per-operation owner
registration is bounded. A 64,000-step failure preserves every host owner/payload
and releases all VM owners. This implementation does not claim constant collection
work beside large owner tables. A future optimization must retain complete-root,
identity, counter, failure-atomicity and prepaid-work guarantees; suppressing that
audit or enlarging this test's budget would not establish the improvement.

Three real-process tests cover both CLI developer modes, pretty/JSON counters,
supplied include packages, discarded unknown names with no retained tables, and
private nested/active-call error locations. developer_blocks.gsc returns integer
20 in default disabled mode and 42 with --gsc-developer enabled.

Complete Windows debug/release workspace passes 424 frontend/273 legacy tests,
seventeen private script tests, two script doctests and all three new process
tests. All-target Clippy, denied-warning rustdoc, formatting and prior inventories
pass. Seven native generators, 56 admissions, 98/59/31/38 local/switch/function/
package mutations, 99 structural replays, twelve suite-policy regressions and
diff/repository policies pass (1106 sources/eight crates/largest 287 lines).
Linux/hosted CI remain unrun. General
ownership/nested Array edges, entity/thread methods, scheduling, engine builtins,
full stock/mod GSC compatibility and joinable gameplay remain open.
