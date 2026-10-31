# ADR 0017: Typed legacy module declarations above structural syntax

Date: 2026-10-31

Status: accepted and implemented; workspace validation recorded separately.

## Context

A concrete grammar tree retains exact reductions but does not expose function
signatures or include/directive ownership. Compiler phases need typed declarations
without repeating native parse-object allocation or interning through fixed pools.
The source actions fold function/parameter identifiers and construct linked native
lists; those storage and publication choices must remain separate from syntax.

## Decision

Lower a complete legacy Script tree into LegacyModuleSyntax with ordered include
lexemes and top-level items: functions, animation-tree directives, and developer
begin/end markers. Each function owns only a bounded vector of borrowed parameter
lexemes and borrows its original statement-list node. Keep source names/case,
duplicates, directive literals, and developer markers unchanged. This phase does
not resolve imports, fold/intern names, check duplicates/balance, decode directive
strings, lower body expressions, compile, or execute GSC.

Scan the flat tree in construction order; the pinned grammar emits each declaration
after its body while retaining declaration order. Parameter lists are lowered
iteratively and reversed into source order. No recursive walk or native pointers
are needed. Reject Fragment entry explicitly. Verify required child shapes and
identifier terminals before publication; return no partial module on failure.

Bound item/include/parameter counts, node visits, and retained logical metadata
before fallible vector growth. Metadata includes item records, borrowed include
lexemes, and parameter elements; it excludes source/tree ownership, spare capacity,
allocator overhead, and stack-local builders. Wide counts/storage do not inherit
old string-pool, packed-handle, or bytecode parameter widths. No runtime function
IDs or wire representation are assigned by this syntax stage.

Guard the pinned semantic dispatch index, action-table shape, relevant production
lengths, and native action instructions during syntax-table verification. These
checks ground declaration roles without executing native actions. Future binding
must implement verified name/developer/scope semantics independently; raw spelling
here must not be mistaken for case-sensitive legacy name resolution.

The offline inspector also lowers declarations and reports item/function/include/
parameter counts plus separate work/storage accounting. Declaration failures carry
their own phase/category and original locations. No source names/values enter logs.

## Alternatives and migration

Interning during lowering would couple source structure to mutable runtime state
and fix ownership before binding/VM design. Recursive list traversal would depend
on source depth. Re-scanning function text would lose established grammar roles.
The bounded typed views can feed later binding and body lowering while retaining
the original tree for diagnostics; no engine state is migrated or activated.

## Evidence and limits

Six tests cover typed includes/functions/parameters/directives/developer markers,
source spans/borrows/order/case, duplicates, explicit Fragment rejection, every
successful structural Script replay, privacy, and exact/one-less budgets. Synthetic
input retains 66,000 function declarations and 1,024 ordered parameters. These
prove frontend representation only, not executable function/argument capacity.
Statement-list spans retain empty-production anchors and may include leading
trivia; they are concrete source spans, not trimmed semantic statement spans.
Full semantic AST/binding, original compiler traces, stock-mod execution, bytecode,
scheduler/VM, and expanded language/services remain open acceptance gates.
