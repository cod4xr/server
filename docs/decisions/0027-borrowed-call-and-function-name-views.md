# ADR 0027: Borrowed call and function-name views

Date: 2026-10-31

Status: accepted; implementation and validation tracked separately.

## Context

Primary calls and function references still contain raw subtrees. Compiler binding
needs original local/external names, indirect callee expressions, immediate/thread
syntax, optional method receivers, and argument lists. Constructing these views
must not resolve modules, intern names, allocate runtime IDs, evaluate receivers,
create threads, or enable builtin/host services.

## Decision

Add bounded local borrowed function-name and invocation views. Names retain exact
module/path and identifier tokens for later canonical identity/resolution. Calls
retain immediate/thread mode, optional unresolved receiver, named/indirect callee,
original argument-list syntax, and the supplied source node/span. Indirect targets
remain expressions rather than guessed names. Recognize both direct call/name
productions and their existing primary wrappers.

Construction performs a constant number of checked child reads without allocation,
recursion, source copying, or descendant traversal. Ordered argument lowering is
an explicit separately budgeted operation using ADR 0025, not hidden constructor
work. Source order does not define argument/receiver evaluation order. Names are
not filesystem paths to open or network endpoints to access; binding/host policy
remains separate. Errors/Debug omit source bytes and private names.

## Alternatives and consequences

Flattening a callee into a string loses indirect expressions and source identity.
Resolving names here couples syntax to module ownership and host capabilities.
Allocating arguments implicitly conceals work/storage admission. Treating thread
spelling as an OS-thread action would choose scheduling semantics prematurely.
These responsibilities remain explicit compiler/runtime work. Local views are
implemented structure, not runnable functions, linked modules, or legacy execution.

## Evidence

Guard named/function-reference, callee-target, immediate/thread, and function/method
production lengths/nonterminals and action roles in pinned source. Tests must
cover local/external/indirect targets, both invocation modes, optional receivers,
exact name/case/spans, empty/nested arguments with explicit budgets, function
references, category failures, independent trees, long names, and private diagnostics.
Source profiles do not execute native semantic actions or prove compiler/VM parity.
