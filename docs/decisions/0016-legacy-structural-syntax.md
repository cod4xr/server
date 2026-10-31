# ADR 0016: Bounded legacy structural grammar trees

Date: 2026-10-31

Status: accepted and implemented; workspace validation recorded separately.

## Context

Raw lexemes and decoded literals do not describe function/control-flow structure.
The pinned repository contains a generated native parser, but no editable source
grammar was found. Its semantic actions allocate native parse objects, intern
strings, and depend on compiler globals. Those actions cannot become a runtime
dependency or silently constrain the expanded GSC implementation.

## Decision

Recover the signed LALR action/production/goto/terminal-translation data directly
from the pinned assembly and validate table dimensions, padding, entry productions,
and dispatch constants. Implement an iterative Rust structural driver with a
growable state stack and a flat concrete syntax tree. Do not execute native semantic
actions, mutate interning/compiler state, or infer a semantic AST from counts.

The script loader injects a plus selector before source. Inject that control
terminal with an explicit zero-length span rather than copying it into source.
The grammar's star entry also accepts supported expressions/statement fragments;
call it Fragment without claiming an original debugger/evaluator execution trace.
The entry production is the tree root; a separately shifted EOF node is retained
in construction-order iteration. Root spans need not include trailing trivia.

Retain original terminal lexemes and source bytes. Trivia is available through
source spans, not extra child nodes. Empty productions have empty anchored spans;
ordered child references only point to earlier private arena entries. Public views
borrow their tree rather than exporting forgeable or cross-tree integer IDs.
Construction and destruction do not recurse through script nesting.

Admit node, edge, live-stack, logical metadata, and grammar-action budgets before
fallible growth. Scanner work has its own budgets; source indexing is admitted
separately. Logical metadata includes node/edge/live-stack storage during parsing,
excluding spare capacity, allocator overhead, and process RSS. These limits are
configurable policy, not old native fixed stack or packed-ID capacities.

Respect default reductions before fetching a new token and retain EOF lookahead
after shifting it. Fail at the first needed scanner/syntax/resource error without
publishing a partial tree. Error recovery intentionally differs from the original
compiler. Debug/errors expose categories, byte offsets, and counts, not source.

## Alternatives and migration

Linking native assembly would preserve architecture/global-state assumptions.
Inventing a new legacy grammar without source traces would risk syntax drift.
Recursive boxed trees would couple destruction to adversarial source depth.
Scanning/decoding/interning everything first would publish premature semantics
and change failure ordering. The flat structural phase is independently useful to
later semantic lowering/tooling; binding, values, module publication, verified
bytecode, scheduling, execution, and modern language/services remain separate work.

## Evidence and limits

Ninety-nine independent Python source-table replays cover script/fragment entry,
includes, control flow, vectors, calls, statements, prefixes, EOF, nested groups,
multiple functions, and lexical-versus-syntax failure ordering. They compare exact
successful reduction sequences but do not execute the original parser or semantic
actions and do not increase the original-C fixture count.

Seven Rust tests additionally cover every two-byte input, original borrowed spans,
ordered children, literal-stage composition, privacy, exact/one-less budgets, and
12,000 nested parentheses with iterative teardown. Adversarial testing found the
additional statement-fragment entry forms; extraction now verifies all four entry
productions. Structural success deliberately does not validate integer ranges,
names, types, builtin availability, or executable GSC semantics. Stock-mod,
original-parser trace, full compiler/VM, and modern feature acceptance remain open.
