# ADR 0053: Explicit offline primitive GSC execution

Date: 2026-10-31

Status: accepted; implementation and validation tracked separately.

## Context

The CLI inspects GSC structure and bindings, while compilation/execution is only
available through the library/tests. A concrete offline entry point lets operators
exercise implemented language features, source locations, limits and readable
development diagnostics. It must clearly identify the primitive subset and never
pretend unsupported imports or engine services have been resolved.

## Decision

Add --execute-gsc PATH as an explicit offline mode, separate from --inspect-gsc.
Reuse bounded snapshot/index/parse/declaration/binding phases and developer-mode
selection. Reject network-only options and conflicting modes. Execution requires
an active canonical local main definition; never select index zero as a fallback.
Reject include/animation-tree directives until resolution is implemented. Compile
every active local body under default module/function/kernel budgets before any
invocation; disabled declarations retain their existing semantics. Invoke main
with the existing empty script-argument prefix, so missing parameters are initialized
Undefined. Reuse default global fuel/frame/setup/count/byte admission without game
state, external I/O builtins, plugins, unbounded waits or native runtime dependencies.

Keep operational events low-cardinality and sanitized: phases, located fixed
errors, compiler/runtime work and peak admitted resources, never source text,
function spelling, host paths, return contents or argument values. Explicit result
data goes to stdout only: a small typed primitive result, including vector values;
function results reveal only the category. This operator output is not a client/
plugin/serialization ABI. Handle stdout I/O failures instead of panicking.
Development stderr remains readable/colorful by default; JSON flags/files retain
their existing contract and flush lifetime. Include original source locations for
compilation and runtime failures without printing private code.

## Evidence and limits

Run real process tests for successful calls/references/builtins/switches, all result
categories, canonical/developer main mapping, missing/main-disabled entries,
unsupported directives/syntax/services, compile/runtime/fuel failures, privacy,
JSON/human logs, offline/conflicting arguments and existing inspection/query modes.
Demonstrate a bounded local example with the normal target directory after stopping
the verified old development executable. Preserve complete workspace gates.
This executes the supported primitive subset only. Imports, heap values, string
switch, scheduling, game services, full mods, a joinable server and Linux remain
open. No dependency, speculative framework, alternate target or deployment.
