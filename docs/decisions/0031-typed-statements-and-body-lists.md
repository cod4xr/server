# ADR 0031: Borrowed statements and admitted ordered body lists

Date: 2026-10-31

Status: accepted; implementation and validation tracked separately.

## Context

The restricted scalar compiler supports only empty/single-return bodies. General
functions need checked statement roles and complete ordered bodies before local
binding or control-flow lowering. Native compiler local/call/value capacities must
not become new storage ceilings. A structural node does not establish valid scope,
an lvalue, event argument semantics, control-flow legality, or host authority.

## Decision

Expose allocation-free local borrowed statement views for assignment/update,
return/wait, call, event, profiling/debugger, block/developer block, conditional,
loop, switch/label, and empty-statement productions. Strip only pinned bounded
statement wrappers and preserve the supplied node/span. The direct statement node
remains available for operation source maps; the restricted scalar compiler reuses
this classification without broadening its profile. Keep child expressions,
references, bodies, event-specific argument lists, for-condition/update options,
and profile labels unresolved. Do not coerce distinct event payload productions
into the ordinary expression-list profile. Scope, values, lvalues, break/continue
legality, execution order, developer activation, and scheduling stay separate.

Lower body-list productions 109/110 into a flat borrowed source-order index.
Preflight both immutable spine passes, count, address arithmetic, and logical
node-index storage before fallible allocation. Use configurable count/work/byte
budgets, wide counts, iterative collection/reversal, and flat drop. Nested bodies
remain separate nodes and are never recursively flattened. No complete-body
statement validation or execution follows from retaining a list.

Views and indices contain scoped immutable nodes; they do not own source or
runtime state. Debug/errors retain spans/counts/fixed categories without source
bytes, labels, identifiers, or paths. Failure never publishes a partial list or
changes the source. Wrapper classification is bounded and allocation-free;
descendant walks need independent budgets.

## Alternatives and consequences

Reading raw production numbers throughout binding/code generation would duplicate
unchecked assumptions. Eager recursive body lowering would entangle local shape,
storage, scope, and runtime semantics and retain native-depth risks. A single
generic list abstraction would obscure different body/expression/event profiles.
These concrete views keep those responsibilities explicit and leave compilation
and execution of general bodies as required subsequent work.

## Evidence

Guard the pinned lengths/nonterminals/action constructors and compound-opcode
identities. Test every statement role, wrapper, child order/span, opaque optional/
event payload, independent live trees, privacy/category failures, nested and wide
body lists, count/work/storage boundaries, and iterative drop. Source-derived
profiles/replays do not run the original compiler or establish native VM parity.
