# ADR 0046: Undefined comparisons in the primitive execution profile

Date: 2026-10-31

Status: accepted; implementation and validation tracked separately.

## Context

The primitive kernel represents Undefined, but every comparison currently routes
through numeric/vector conversion and rejects it. The pinned Scr_EvalEquality
returns integer one for two undefined values, and Scr_EvalInequality negates it.
Scr_CastWeakerPair rejects undefined paired with a number/vector before comparison;
even two undefined values cannot be ordered. Rust enum equality is not GSC equality.

## Decision

Give comparisons their own focused kernel dispatch leaf. Preserve the existing
number and vector comparison helpers, including scalar tolerance, vector exact
equality, mixed numeric broadcast and unsupported vector ordering. Two Undefined
values compare equal/not-unequal. Any other comparison involving Undefined fails
with a fixed UnmatchingTypes category, without retaining operand contents. Numeric
arithmetic and branch truth rules do not change. Use the existing instruction,
proof, fuel, source maps and both source compilation profiles without new storage.

Execute the actual pinned coercion/comparison C helpers with Undefined paired
in both orders with integer/float/vector boundary cases. Keep the existing bounded
vector pool/ref/error stubs and explicit FP profile; they do not establish native
reference cleanup, errors or complete VM equivalence. Retain exact success/error
counts, source missing-argument/initialized-Undefined cases, lazy evaluation,
source locations, error privacy and quota regression coverage.

## Alternatives and consequences

Treating every mixed undefined comparison as false would change the pinned helper
behavior. Reusing derived PartialEq would change numeric/vector rules. A general
value/coercion framework would add unused machinery before heap values exist.
The dispatch leaf instead has one implemented responsibility and can later gain
separately evidenced value categories. No object/string equality, builtin
isDefined, native frame/error unwinding or full-GSC acceptance is supplied here.
