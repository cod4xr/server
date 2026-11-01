# ADR 0044: Primitive source vector construction and explicit component order

Date: 2026-10-31

Status: accepted; implementation and validation tracked separately.

## Context

Owned vectors and the primitive kernel now have original-helper evidence and
budgeted execution. Parenthesized source still accepts only a singleton. The
native parser prepends expression nodes, its compiler walks that linked head,
and the vector VM/helper consume reversed slots. Component order must be explicit
before source triples are accepted; native constant folding/diagnostics are not
reproduced by the primitive compiler's invocation-only validation contract.

## Decision

Keep PureScalarReturn unchanged. Extend PrimitiveStatements to parenthesized
triples, while retaining singleton grouping and rejecting empty/two/four-or-more
groups. Bind the source contract once in the function builder, rather than
inferring vector permission from whether a local table happens to be present.
Use that same profile for publication/validation and result tagging.

Lower exact three-element list shapes through a focused group leaf with bounded
fixed child probes, charged work and existing admitted flat tasks. Evaluate z,
then y, then x, and emit MakeVector consuming bottom-to-top z/y/x. This preserves
the selected linked-list traversal/slot roles without native fixed temporary
arrays or a recursive owned IR. Keep existing scalar/vector operators, source
locations, numeric conversion, initialized-name checking and runtime type errors.
Map construction to the original group span; do not claim native error-index/
cleanup parity. Primitive compilation still does not pre-execute fabricated
arguments or emulate native compile-time constant folding/error order.

Verify/hash-lock the exact pinned parse-tree helper before adding source guards.
Guard prepend operand/link roles, grammar/action/list/group roles, selected
compiler group count/traversal/emission and VM/helper stack layout/opcode identity.
These are selected source-shape evidence, not original parser/compiler/VM execution
or complete source side-effect/evaluation/constant-folding conformance.

## Evidence required

Execute actual source vector returns, locals/assignments/compound operations,
scalar broadcast/equality distinctions and control-flow/loop paths. Check signed
boundaries/float bits, nested grouping, invocation errors and original spans,
right-to-left failing-component order, uninitialized reads, caller isolation and
redaction. Preserve both explicit profiles and reject unsupported group shapes/
values even in skipped source. Cover exact/one-less compiler/literal/kernel
limits, wide vector locals and deep flat lowering/failure cleanup. Regenerate all
pinned oracles after adding the parse-tree hash; retain native/full-GSC/Linux
acceptance gates as open.
