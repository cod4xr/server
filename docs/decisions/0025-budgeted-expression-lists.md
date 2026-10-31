# ADR 0025: Budgeted source-ordered expression lists

Date: 2026-10-31

Status: accepted; implementation and validation tracked separately.

## Context

Parenthesized primaries and invocation syntax share legacy expression-list
productions. A compiler needs their complete source order without recursive
ownership, native linked nodes, fixed argument arrays, or copying source text.
The original semantic actions prepend arguments; source order must not be confused
with native runtime evaluation/push order. Arity/value/binding rules are later work.

## Decision

Lower the optional/list productions into a fallibly allocated Vec of scoped,
borrowed expression nodes. Preserve source order and allow double-ended iteration.
Accept the optional wrapper, empty production, nonempty list, and its prefixes;
reject unrelated nodes. Retain the original supplied syntax/span. Do not decode
values, flatten nested expressions, decide group/vector/call arity, bind targets,
evaluate arguments, or choose runtime parameter storage.

Preflight iteratively before allocating output. Configurable u64 budgets admit
expression count, both passes' node visits, and logical retained node-index bytes.
Admission uses checked arithmetic; reserve is fallible, and no partial list is
published. Two immutable passes walk the left-recursive spine and reverse the
staged index into source order. Work is linear and bounded by admitted node visits
and count. Logical metadata excludes the source/CST, Vec spare capacity, allocator
overhead, and local builder fields; it is not RSS or aggregate runtime heap usage.

Errors expose categories, resource counts/budgets, and the original supplied
list's byte offset. Source strings/names/paths remain excluded. Empty lists require
no index storage. Configured large budgets remain subject to address space and
fallible allocator admission, never a promise of unlimited memory.

## Alternatives and consequences

Native prepend order leaks evaluation policy into a frontend container. Recursive
lists risk stack exhaustion. Growing output before checking the complete budget
retains avoidable partial allocations on resource failures. A borrowed reverse
iterator alone cannot provide repeated source-order iteration without extra work
or storage. The admitted flat index gives concrete compiler-facing ownership while
leaving runtime call/parameter semantics explicit and unimplemented.

## Evidence

Guard pinned production lengths/nonterminals and prepend/alias/empty action roles.
Tests need exact order/spans, empty/single/nested forms, all resource boundaries,
unrelated-node failures, private diagnostics, independent sources, and admission
beyond old narrow counts. This proves frontend representation, not executable
argument capacity, compiler/VM behavior, stock-mod compatibility, or native action
execution. Caller compilation must independently verify argument evaluation order.
