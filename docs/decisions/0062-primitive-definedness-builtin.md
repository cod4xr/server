# ADR 0062: Definedness for the implemented value subset

Date: 2026-11-01

Status: accepted for primitive values; entity/thread lifetime support remains open.

## Problem and decision

GScr_IsDefined differs from the four exact-one type predicates. It directly calls
Scr_GetType(0), whose checked access rejects a missing first parameter and accepts
additional supplied parameters. For nonpointer values it returns whether the type
tag is nonzero, not numeric truth. Its pointer path also rejects removed entities
and parent types above VAR_ARRAY. Do not infer definedness from truthiness or
silently give it the exact-one arity policy used by isint/isfloat/isvector/isstring.

Add sealed KernelBuiltin::IsDefined and bound-source fallback after active local
function lookup. Return false for Undefined and true for every other currently
implemented primitive category, including zero/NaN, empty plain/localized strings,
vectors and function references. Require at least one supplied argument; evaluate
every supplied argument rightmost-first, use the first source argument, and discard
all extras. Preserve local/case/developer shadowing, skipped-call timing and full
call spans. No entity/thread values are fabricated to complete the builtin.
Exhaustive value and arity matches force an explicit policy when future value
categories or builtins are added.

The verified stack effect already consumes the complete supplied count. Discard
exactly that count, retaining older stack prefixes. Every operand required an
earlier admitted push, so aggregate discard work remains bounded by global fuel
and stack count/bytes. No temporary argument array, extra frame, per-call work
reset or native byte-count ceiling is introduced. The four existing predicates
keep their exact-one behavior and unchanged native C corpus.

## Evidence boundaries

The newly pinned _g_scr_main_mp.asm canonical SHA matches its Git blob at the
existing upstream revision. Selected definedness branches, first-parameter access,
result writes, registration and tag roles reject 46 mutations. Isolated x86
test/setnz/movzx and signed compare/jump instructions produce 351 O0/O2 outcomes
with modeled first-parameter admission. This does not execute the original
function, calling convention, parameter/error service, object lifetime or VM.
Raw comparator parent words are not valid allocated native objects.

56 fixture rows run through verified Rust execution: 49 results/seven missing-
argument failures across seven implemented categories and bounded counts through
65,536. The remaining 295 rows explicitly stay instruction/helper-only evidence
for unavailable tags/parent types and UINT32_MAX counts. Separate exact/one-less
fuel/stack/byte tests preserve an older stack prefix under 65,536 operands.
Four source tests cover optional parameter Undefined, zero/empty values, functions/
vectors/localized tags, evaluated extras, runtime/skipped arity, maps, local/case/
developer fallback and all 256 extra string literals being evaluated. A real
process test covers optional/excess/missing arguments and private located failures.

The runnable defined_demo.gsc returns integer 1 for a missing optional parameter.
Source reads still obey the current initialized-local profile; this change does
not make unbound names, entity fields, arrays or other unsupported expressions
executable. Full legacy binding/lifetimes, general builtins/services, scheduling,
native unwinding, stock mods and Linux acceptance remain open.

## Alternatives and migration

Treating Undefined as numeric zero would misclassify defined zero values. Enforcing
exact-one arity would reject accepted native calls. Accepting zero arguments as
Undefined would hide a native parameter-access failure. Keep these separate from
future modern optional/variadic APIs. Existing local isdefined functions continue
to shadow fallback, while previously unknown unshadowed calls now use this sealed
subset. Future object support must add verified lifetime checks before activation;
raw handle existence cannot stand in for native entity/thread definedness.

## Validation

All required Windows workspace policy/formatting, warnings-denied Clippy/rustdoc
and debug/release tests pass: 243 frontend/142 legacy tests and the new process
test, 666 sources/eight crates, largest source 287 lines. The full existing vector,
index, Undefined, function-reference, predicate and string-value corpora regenerate
unchanged and now run in the upstream CI job. The definedness corpus/46 mutations
and 99 structural replays pass. CI is configured but has not run on a hosted
repository; Linux/full-mod acceptance remains open. Wide typed/source counts are
capacity expansion, not native byte-encoded caller acceptance.
