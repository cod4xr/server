# ADR 0058: Legacy plain-string type predicate

Date: 2026-10-31

Status: accepted for the plain-string predicate subset.

## Decision

Add sealed isstring to existing bound builtin fallback and typed instructions.
The pinned GScr_IsString requires exactly one parameter, reads its type only after
the arity check, and calls Scr_AddInt with a comparison against VAR_STRING (2).
It returns false for VAR_ISTRING (3). Preserve that distinction rather than
guessing localized-string semantics. Keep ordinary local/developer shadowing,
rightmost argument evaluation, runtime arity and unknown-service boundaries.

Extend the actual C predicate oracle with the extracted function, integer result
service and complete enum/count matrix at O0/O2. Guard exact registration, type,
arity-before-read, result-service and selected compiler roles with mutations.
Replay representable plain-string tags through checked heap arguments, never
runtime string constants; unavailable tags and UINT32_MAX counts stay helper-only.
No new native/plugin callback or runtime dependency is introduced.

## Validation

Require actual native fixtures and guard mutations, argument/type categories,
source literals/locals/calls, local/developer shadowing, arity/evaluation failures,
source locations/privacy, skipped calls and existing wide argument/resource gates.
Localized runtime values, isdefined object lifetime, general standard library,
host/game services and full native VM/mod acceptance remain open.

## Evidence

Actual extracted isint/isfloat/isvector/isstring functions produce 832 O0/O2
outcomes. Of these, 168 run through checked Rust argument/local/instruction
execution: 24 successful results and 144 exact-arity failures. Another 664 rows
remain helper-only for unavailable tags and UINT32_MAX counts; localized tag 3 is
not a Rust runtime value. The helper records its false isstring result without
claiming localized-value execution. Source-role guards reject 46 mutations.

Five source tests cover case/type categories, literal and allocated strings,
local/developer shadowing, skipped bad arity, call maps, rightmost argument errors,
privacy and evaluation of all 256 supplied string arguments before arity failure.
The process string test also calls isstring before output. Existing typed fixtures
exercise 70,000 supplied handles without copying their payload or truncating arity.
No native compiler/VM/interner/ref cleanup or plugin ABI is exercised.
