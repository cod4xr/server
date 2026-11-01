# Assessment 0093: Literal-constant loop scopes

Date: 2026-11-01

Status: selected Windows evidence accepted; Rust physical-scope implementation
and source indexed writes remain open under ADR 0088.

Eight selected cases now combine the pinned x86 primitive evaluator/planner and
loop/branch/slot/transfer emitters with the actual pinned C Scr_CastBool helper.
Six literal-true cases execute emission; two literal-false cases execute planning
only. Independent complete state/request/callback expectations agree with both
C driver optimization levels. No false-loop emitter/diagnostic outcome is claimed.

## Boundary

The runner can compile an additional extracted C translation unit inside its
existing checked temporary directory. A small explicit word-sized value/type
prelude supplies the bool helper's ABI; unsupported diagnostics fail. The actual
bool function body is extracted only after revision/hash verification and is not
rewritten. Integer literals use the actual x86 EvalPrimitiveExpression. The
modeled expression dispatcher asks that evaluator instead of inventing a constant
result. Nonreference cleanup callbacks permit only Undefined/float/integer tags;
reference-bearing cleanup fails. Each case observes one scalar cleanup callback.

These cases contain only literal integer zero/one conditions and dynamic variable
conditions in nested branches. Float/binary/general constant expressions, reference
values, error unwinding and false-condition emission remain outside the profile.
Statement dispatch/string IDs/opcode output/temp storage remain modeled. No native
parser/VM/value freeing/opcode optimizer/alignment claim follows. Windows passes;
Linux/hosted CI is unrun. Previous 34/36/10/9 scope fixtures remain byte-identical.

## Consequences

A true loop with a single assignment-before-break can export that local, unlike
the earlier dynamic-condition counterpart. Disjoint break branches do not export
their private names merely because the loop is constant true. Continue/fallthrough
declarations and ordinary exiting facts therefore need separate treatment. For
loops additionally retain their distinct initializer/body/update declaration and
initialization states. A single union or intersection across every control edge
cannot describe all these results.

The true mixed-exit cases retain separate initialized bits even when both names
have physical public slots. False planning snapshots do not materialize owners
and are explicitly marked analysis-only in the fixture and validator. A populated
declaration table is not evidence that a cell contains an initialized array alias.

Seven admission regressions cover case completeness, false-case boundaries,
preserved single-break names, private disjoint-break cleanup, mixed-exit bits and
scalar cleanup accounting. The five scope admission suites now contain 32 tests.
All five native scope generators, existing local/switch role mutations, structural
replays and repository policy remain required; no Rust behavior changes here.
Complete Rust workspace acceptance remains ADR 0088.

Next, implement the iterative metered Rust declaration/public/physical-lifetime
plan for reviewed source profiles. Keep current initialization facts separate,
account ordinary/fallthrough/return/break/continue paths explicitly, release aliases
on every supported edge and test later COW/key order plus exhausted cleanup and
deep/wide budgets. Extend or explicitly reject unsupported constant/developer
profiles before writable source activation. Full GSC, scheduling, general nested
ownership and join gates remain open.
