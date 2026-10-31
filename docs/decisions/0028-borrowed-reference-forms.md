# ADR 0028: Borrowed reference forms without premature binding

Date: 2026-10-31

Status: accepted; implementation and validation tracked separately.

## Context

Primary references still contain raw subtrees. Binding needs original identifiers,
field/index operands, dollar-prefixed names, and the reserved .self production.
The recovered grammar distinguishes object.self from object.Self; folding names
before preserving that role would lose source information. Structural references
are not automatically writable, valid objects, or resolved variables.

## Decision

Provide a constant-cost borrowed reference view over direct reference productions
and existing primary wrappers. Keep name tokens, unresolved object/index operands,
and source spans without source copying, interning, recursion, allocation, lookup,
mutation, or evaluation. DollarName and SelfMember describe source forms; they do
not invent runtime/debugger authority or equate reserved syntax to a generic field.

Binding owns canonical identity, scopes, object/collection types, availability,
mutability, and diagnostics. Runtime storage owns wide handles and budgets. The
view chooses none of those representations. Arbitrary descendant walks remain
caller-bounded; each local view reads only a constant number of checked children.

## Alternatives and consequences

Flattening a reference into a string loses index expressions and nested objects.
Treating every form as a writable local breaks fields and reserved syntax. A
recursive owned tree duplicates the flat CST and risks deep-input stack growth.
The local view preserves exact source roles while making unresolved semantics
explicit. It is not an lvalue validator or an executing variable/object system.

## Evidence

Guard the five reference production lengths/nonterminals and action constructors.
Tests need every role, exact operands/spelling/spans, .self versus .Self, direct/
primary wrappers, nested references walked iteratively, category failures, private
diagnostics, and independent sources. Source-table/action checks do not execute
native compiler/VM code or establish actual field/dollar/self semantics.
