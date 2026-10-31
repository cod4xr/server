# ADR 0030: Explicit pure scalar function source-compilation profile

Date: 2026-10-31

Status: accepted; implementation and validation tracked separately.

## Context

Typed frontend views and a verified instruction executor still do not compile GSC
source. The next concrete integration is a deliberately bounded profile: a
zero-parameter legacy function with an empty body, return, or a single scalar
return expression. It must produce real executable code/source locations, preserve
short-circuit control flow, reject unsupported syntax, and publish no partial
function. It is not whole-module/stock-mod compilation or a complete GSC VM.

## Decision

Compile from an existing typed LegacyFunctionSyntax into an owned scalar program
and original per-instruction byte spans. Support numeric/undefined/bool literals,
single-value parentheses, numeric infix/prefix operators, and lazy logical OR/AND.
Empty/void-return functions produce actual Undefined. Reject parameters, multiple
or other statements, strings/vectors/objects, references, calls, and host features
explicitly. Callers retain module/source association and own prior declaration/
developer/name binding policy. No module is activated by compiling one function.

Lower iteratively with a quota-accounted task stack. Private unresolved branch
records are compiler relocations, not dummy valid kernel instructions. Bind each
forward logical branch once, resolve all records, verify code, and perform bounded
pure validation execution before returning the owned function. Numeric/type errors
therefore fail compilation with mapped source spans. No optimization is implied;
the emitted operators still execute when the resulting function is invoked.
Unreachable unsupported source remains unsupported in this restricted profile.

Numeric token conversion remains a separate bounded phase. Preserve float sign
bits, including negative zero. Check a negative integer's whole signed domain so
-2147483648 is admitted mathematically; unsigned 2147483648 remains rejected. The
native atoi overflow for that magnitude is undefined and not used as evidence;
this is an explicit domain policy, not a claim about every CRT's overflow result.

## Ownership and resources

Admit compiler task actions/literal bytes/resolution work, instruction count, live
task frames, and logical compiler metadata with checked arithmetic and fallible
growth. Budgets separately cover literal conversion, kernel verification, and
validation execution. Compiler metadata includes pending code/task records and
overlapping resolved code/source-map storage, excluding source/CST, vector spare
capacity, allocator overhead, and kernel-owned/verifier/stack storage accounted by
their separate budgets. These scopes do not establish aggregate RSS accounting.

The result owns code/maps and retains no source/tree references or raw names.
Errors include original byte spans, fixed categories, compiler work/metadata, and
sanitized nested literal/kernel failures. Source text, filenames, paths, values,
and private identifiers remain outside Debug/errors. Failed stages drop private
staging without publishing a callable function or changing external state.

## Alternatives and consequences

An eager boolean lowering would execute skipped errors. Publishing code before
verification/validation would expose invalid functions. Parsing source directly
inside execution would entangle ownership and resource limits. Treating unsupported
features as undefined/zero would fabricate support. This profile instead exposes
its limits and keeps full functions, locals, calls, objects, scheduling, host services,
modern language, native diagnostic ordering, and stock-mod acceptance open.

## Evidence

Guard return/body/wrapper roles against the pinned grammar/actions. Test the whole
source/index/lexer/parser/declaration/compile/verify/execute path, source locations,
operator/short-circuit results, signed boundaries, privacy, source-independent
ownership, every resource boundary, deep iterative input, and rejection of unsupported
syntax. Source-table profiles do not execute the original compiler or establish
complete GSC compatibility. Linux and broader release gates remain separate.
