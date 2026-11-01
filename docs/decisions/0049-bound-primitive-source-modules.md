# ADR 0049: Source calls within a bound primitive module

Date: 2026-10-31

Status: accepted; implementation and validation tracked separately.

## Context

Primitive functions compile in isolation and verified inventories execute typed
calls. Source calls still have no executable link between these layers. Local
LegacyModuleSymbols already validate developer regions and canonical active name
identity, but explicitly leave call policy unresolved. Extending isolated function
compilation with fabricated targets or running each callee independently would
lose forward references, shared fuel, source maps and atomic publication.

## Decision

Add a separate bound primitive-module compiler taking an existing local symbol
index. Keep the public isolated function profiles unchanged. One private lowering
mode owns syntax/context permission: isolated pure return, isolated primitive
statements, or primitive statements with a borrowed immutable local target scope
and caller visibility. Reuse statement/expression/local/control-flow lowering;
do not clone the complete target table per function or widen every helper with
an extra unused lifetime. The borrowed tree can be reborrowed for the scope's
shorter lifetime; no source/name/symbol borrow may escape in the compiled result.

Build the complete active target inventory before lowering any body. Resolve
unqualified named immediate calls by existing legacy identity;
support forward, recursive and mutual calls. Reject external paths, receivers,
indirect/thread calls and unresolved names explicitly. Do not invent builtin/host
fallback or process imports/animation directives through a local-symbol-only API.
Preserve developer-disabled declarations as nonexecutable metadata. Ordinary
functions cannot reference developer-only functions; developer callers may call
ordinary/active developer definitions under the supplied binding mode. Guard the
selected native resolved/deferred visibility roles without claiming complete
native compiler/linker execution or diagnostic precedence.

Lower arguments rightmost to leftmost through admitted flat tasks, tied to pinned
prepend/list/caller traversal roles. Count/preflight list shape and argument quota
without an unaccounted temporary argument array. Emit the typed Call only after
argument instructions; keep runtime excess-argument checks from ADR 0048. Invocation
statements discard the result, while expression calls retain it. Preserve compiler
work, initialization checks, whole-call/descendant spans and existing unsupported
statement/for-clause policies. Restricted source checks still inspect skipped code;
native constant folding, scopes and complete error order are not established.

Resolve all bodies to owned staged code/layout/maps, then verify the complete
inventory once before publication. No partial source module or activation escapes.
Retain declaration-order inclusion/visibility/parameter/span metadata and an
explicit mapping to active function indices. Callers associate source/module names
and select entries through their original declaration index; the source compiler
does not become a runtime name/interner/module manager. Source maps identify both
function and instruction after source/tree/symbol storage is dropped.

Admit total declarations/active functions, compiler work, instructions and logical
compiler metadata across the entire operation. Include scope key/value payload,
retained declaration/map/staging/definition table records, prior staged code/maps
and the current function's pending/task/binding/snapshot/resolution overlap. Clamp
per-function limits to remaining global admission and translate global exhaustion
explicitly. Kernel proof has its separately admitted whole-inventory scope; its
copied code/verifier payload is additional to staged compiler input, not hidden as
aggregate RSS. Source/CST/symbol ownership, hash controls, spare capacity, allocator
rounding, fixed fields and separately admitted body-index/literal scopes remain
explicitly outside compiler logical payload. Shared FunctionLoweringBudget exposes
code-generation limits without irrelevant standalone proof/validation settings.
Isolated function compilation keeps its public budget fields and projects those
limits; its private mode owns the separate proof/validation configuration.

Empty/all-disabled source inventories retain metadata without fabricated functions.
Allow an empty verified primitive inventory with zero usage; every entry selection
then fails InvalidFunction. This supersedes ADR 0047's empty-inventory rejection;
individual empty typed instruction streams still fail EmptyProgram. No activation,
reload/generation identity, host authority, native/serialized call ABI, scheduler,
strings/objects or full-GSC support is implied.

## Validation

Execute real source examples with forward/mutual recursion, expression/statement
calls, loops, vector results, case identity and missing/excess argument behavior.
Check rightmost argument error/name order, caller isolation, source-independent
ownership and function/instruction maps. Test developer modes/visibility, disabled
unsupported bodies, empty/all-disabled inputs, unknown and unsupported call forms,
later-body failure and all exact/one-less global/per-function scopes. Exercise
source argument/function/local indices beyond native short capacities and deeply
nested calls with flat failed cleanup. Retain isolated compiler/kernel/oracle
regressions, source-role fault injection, Windows gates and explicit open Linux/
native/full-GSC/mod acceptance. No dependencies, empty crates or alternate targets.
