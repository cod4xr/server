# ADR 0024: Borrowed typed primary forms

Date: 2026-10-31

Status: accepted; implementation and validation tracked separately.

## Context

Operator views deliberately leave primitive productions opaque. Compiler consumers
need to distinguish literals, original signed-literal syntax, grouped expression
lists, calls, references, special objects, size access, animation forms, and legacy
debugger forms before resolving/typing their descendants. A structural primary is
not necessarily a constant or a legal runtime value in every execution mode.

## Decision

Provide a separate constant-cost borrowed LegacyPrimarySyntax view. Retain original
source syntax/span and explicit literal polarity; do not concatenate a synthetic
minus into source or decode values during classification. Preserve parenthesized
lists without deciding group/vector arity. Retain call/reference/function-reference
subtrees for their own lowering and name/type resolution. Distinguish empty arrays,
special object roots, size access, named animation, #animtree, and breakon syntax.
The presence of debugger/animation syntax does not enable their runtime services.

The view may compose with the operator view's entry/primitive-wrapper handling.
Nonprimary expressions and unrelated nodes fail explicitly. Read only a bounded
number of children from private immutable CST nodes, without allocation, recursion,
descendant walking, handles, interning, or native pointers. Whole-tree traversal,
literal conversion, list admission, semantic diagnostics, and execution remain
separate responsibilities with their own work/storage/error contracts.

## Alternatives and consequences

Treating all primaries as literals misclassifies calls/references and debugger
forms. Evaluating signed literals in this stage would entangle checked conversion
and legacy overflow policy with syntax. Removing parentheses here loses grouping
versus vector/list information before arity is known. These choices are deferred
explicitly rather than represented as successful execution or invented values.

## Evidence

Check pinned production lengths, action constructors/tags, nonterminal identity,
and relevant terminal spellings through source-table tests. Every primary role,
original child order/spans, signed-literal distinction, independent source trees,
category failures, and private Debug/errors need focused coverage. Profile checks
do not execute native parser/compiler actions; this layer establishes no VM,
debugger, animation system, stock-mod execution, or client interoperability.
