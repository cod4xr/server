# ADR 0023: Borrowed typed expression operators

Date: 2026-10-31

Status: accepted; implementation and validation tracked separately.

## Context

The structural tree preserves parser productions but makes every consumer repeat
grammar numbers and child positions. A compiler needs explicit operator identity,
operand order, source locations, and a distinction between short-circuit logic and
eager operators. Literal decoding and scalar kernels alone provide none of those.
Choosing heap ownership or executing a partial language here would conflate stages.

## Decision

Add constant-cost, allocation-free borrowed expression views above the structural
tree. Classify logical OR/AND separately, sixteen eager infix operators, two prefix
operators, and opaque primary productions. An expression-entry selector and its
single primitive wrapper can be removed during classification; arbitrary statement
or declaration nodes fail explicitly. Retain the original supplied syntax node and
span alongside scoped operand nodes. Child nodes remain tree-owned and unforgeable.

Classification is local: it does not walk descendants, decode literals, bind names,
choose overloaded operand semantics, evaluate values, or imply full semantic AST
validation. Primary nodes explicitly await typed literal/call/reference/vector and
other lowering. Add, for example, is a syntax operator rather than an instruction
to coerce its operands to numbers. No new runtime IDs, allocations, caches, mutable
globals, recursion, or traversal budgets are necessary for a constant number of
node/child reads. Consumers that walk whole expressions must bound that work.

Short-circuit branches remain ordered syntax operands, never prematurely lowered
to eager scalar operations. Later control-flow compilation owns evaluation order
and reachability. Existing literal/string/number helpers stay independently usable.

## Alternatives and consequences

Consumers switching on production integers duplicate compatibility policy. A full
owned expression arena would duplicate the CST before primary/scope/value ownership
is implemented. An evaluator handling only convenient forms could misrepresent
supported GSC execution. This view gives implemented compiler-facing structure
without those claims. Complete primary/statement lowering, binding, compilation,
verified bytecode, and execution remain open.

## Evidence

Guard production lengths and semantic-action roles in the pinned grammar source,
separately from executing those native actions. Tests should cover every operator,
precedence/associativity, exact original spans/operand order, accepted primary
opacity, rejection of other node categories, deep input without recursive walks,
independent trees, and private Debug/errors. Passing these views does not prove
native parser/compiler execution or stock-script conformance.
