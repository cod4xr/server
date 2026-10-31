# ADR 0020: Explicit legacy scalar coercion, comparisons, and division

Date: 2026-10-31

Status: accepted and implemented; full-workspace validation recorded separately.

## Context

Rust numeric operators do not automatically preserve GSC behavior. The source
converts mixed integer/float pairs to binary32, uses approximate float equality,
negates strict predicates for <=/>=, and returns a float for integer division.
These rules are required before expression lowering/constant folding and a VM.

## Decision

LegacyNumericPair represents only two int32 or two binary32 values. Coercion keeps
same-kind inputs unchanged and converts only the integer in a mixed pair. The
selected profile assumes nearest binary32 evaluation with FTZ/DAZ disabled; the
kernel does not set host FP control state or model sticky exception flags.

Implement six explicit LegacyComparison operations returning legacy integer zero/
one. Integer comparisons are exact. Float equality subtracts in binary32, promotes
the absolute difference to binary64, then compares against the source double
0.000001 threshold. A binary32 threshold changes a boundary result and must not
replace it. Inequality negates equality. <=/>= negate greater/less respectively,
so unordered NaN operands return true for both. Equal infinities return false for
approximate equality because their difference is NaN. These are named legacy
semantics; modern numeric rules remain independently versioned future work.

Divide produces binary32 for either input kind, including int32 MIN/-1 without an
integer-division overflow. Integer/float signed zero denominators return the typed
DivisionByZero error. No caller operand is mutated and no partial result escapes.
Source VM default values, error stack/ref cleanup, and unwinding are not simulated.
Finite/infinite results preserve binary32 behavior under this profile. Arithmetic
NaNs assert class rather than payload bits, which Rust does not guarantee here.
Nonarithmetic coercion retains original float bits, including NaN inputs.

Rust's derived PartialEq on LegacyNumber remains Rust data equality; language
execution must use the explicit comparison operation. Pair Debug/errors omit
operand values. The implementation allocates no storage, owns no engine IDs or
heap references, and installs no globals, I/O, or host environment changes.

## Alternatives and migration

Using Rust ==/<= directly would break tolerance/unordered cases. Widening the
subtraction itself to f64 would also change source evaluation. Treating integer
division as integer quotient would change valid scripts. Reimplementing native
VariableValue layouts/refcount/error machinery inside scalar helpers would fix
ownership before full value/VM design. The bounded constant-cost kernel can serve
later compiler/VM code without becoming its object storage or lifecycle model.

Integer add/subtract/multiply/shift overflow policy, modulo edge cases, strings,
vectors, objects, and other full operators remain separate work. Do not infer
their behavior or full GSC execution from these scalar helpers.

## Evidence and limits

The GCC oracle compiles eight actual pinned functions: weaker-pair coercion, six
comparisons, and division. It selects nearest SSE with FTZ/DAZ disabled and asserts
int32/binary32 shapes and FLT_EVAL_METHOD=0. Nonnumeric service stubs abort if used;
error hooks record rather than unwind. Numeric pairs never use stubbed object/
string/vector storage. Actual helper source is extracted verbatim after handling
the source's calling-convention annotations in the test host.

Four Rust tests include 12,416 original cases: 1,552 coercions, 9,312 comparisons,
and 1,552 divisions. Boundary Cartesian products and deterministic generated pairs
cover integer-to-float rounding, tolerance neighbors, signed zeros, subnormals,
infinities, quiet/signaling NaNs, negative values, and zero errors. Focused tests
also distinguish Rust versus GSC equality, float-result integer division, and
paired-value privacy. Division NaN results record class only; no cross-platform
NaN payload, host FP-control independence, full floating ABI, VM/compiler execution,
or stock-mod/gameplay compatibility is established. Linux checks remain separate.
