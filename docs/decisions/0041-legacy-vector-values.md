# ADR 0041: Owned legacy vector value rules before VM integration

Date: 2026-10-31

Status: accepted; implementation and validation tracked separately.

## Context

The kernel/compiler still represents only numbers and Undefined. Legacy vectors
are three binary32 coordinates, but their rules differ from scalar arithmetic:
weaker numeric coercion broadcasts a number; vector multiplication/division are
component-wise; equality uses exact coordinate comparisons rather than the scalar
float tolerance. Addition uses the separate string-aware coercer and rejects
mixed vector/numeric pairs. These distinctions need independent evidence before
changing runtime values or lowering three-element source groups.

## Decision

Implement fixed-size owned, copyable, redacted vector values and explicit vector
operands/coerced pairs/operator/errors in the existing legacy module. Retain
numeric int-to-binary32 conversion and original float input bits. Require at least
one vector at the pair API; scalar-only pairs remain the scalar API's domain.
Keep mixed-add rejection separate from ordinary broadcast, and reject ordering
comparisons. Return a typed divide-by-zero error if any denominator coordinate
is either signed zero, without exposing the native cleared intermediate result.
Exact equality follows coordinate comparisons, including signed-zero equality,
NaN inequality and equal infinities. Arithmetic NaN payloads are not promised.

Inline triples have no allocation/refcount/vector-pool ceiling; they do not prove
VM heap accounting, native cleanup/unwinding or future modern coordinate rules.
Keep scalar kernel values/source compilation unchanged in this step. Source vector
component evaluation/push order, construction opcodes, live stack/local sizing,
budgets and runtime integration require separate evidence and work.

Pin q_math.c after comparing its working bytes with the existing pinned Git blob.
Run selected original C construction/coercion/arithmetic/comparison helpers and
VectorCompare with original math macros in a bounded temporary harness. Stub only
pool/ref/string/error services and document them. Error interception is a harness
boundary, not native unwinding. Use the explicit nearest/SSE, FTZ/DAZ-off profile;
check exact construction/coercion bits, finite arithmetic bits and NaN classes.
Regenerate existing pinned-source oracles after the additional source lock.

## Evidence required

Cover component order/conversion boundaries, scalar broadcasting on either side,
addition's distinct mixed-type errors, component operations and signed-zero
division, exact vector versus tolerant scalar equality, NaNs/infinities, rejected
ordering/scalar-only domains, caller isolation and sanitized Debug/errors.
Keep fixtures/provenance, Windows checks and unsupported runtime/whole-GSC/Linux
gates explicit. No source vector, heap, scheduler or complete VM claim follows
from standalone vector helper parity.
