# Expanded GSC runtime contract

The user requires a substantially richer GSC implementation integrated into the
Rust engine. Ordinary language/service features must not require native CoD4X
plugins. Existing scripts remain a compatibility target. This is a product/design
requirement, not a claim that a compiler, VM, or any listed feature exists yet.

## Language and compatibility

Provide a legacy compatibility mode with independently verified parsing, coercions,
numeric behavior, thread/wait/notify scheduling, errors, builtins, and observable
quirks. Separate compilation from execution; record language/standard-library
versions in module metadata. Do not silently reinterpret old syntax or introduce
names that shadow existing builtins. Preserve existing imports and entry points
where compatibility is claimed.

Add an explicitly versioned modern language mode. Priorities include modules/import
namespaces, first-class functions/closures, richer strings and byte buffers, maps/
sets/growable arrays, structs/enums, optional type annotations, structured error
handling, and clearer iteration/control flow. Final syntax needs an ADR and parser
ambiguity/legacy conformance tests. Preserve legacy integer/binary32 behavior in
compatibility mode; wider numbers must be explicit. Engine handles remain opaque
wide values rather than float IDs.

## Built-in capabilities

| Area | Required direction | Engine boundary |
| --- | --- | --- |
| Data | JSON, binary buffers/codecs, useful collection/string/math APIs | Typed errors and byte/count budgets |
| Modules | Namespaced reusable packages and predictable dependencies | Versioned manifests, deterministic resolution |
| I/O | HTTP, database/storage, virtual files and timers without plugins | Rust host services; asynchronous completion |
| Concurrency | Awaitable services alongside compatible wait/notify threads | Explicit scheduling and cancellation |
| Game APIs | Entities, assets, events and future ApX services | Typed handles and stable host contracts |
| Tooling | Source maps, stack traces, breakpoints, watch/inspection | Compiler/VM metadata rather than native pointers |
| Operations | Profiling, script diagnostics, test/replay tools | Bounded logs, per-module accounting and metrics |
| Reload | Development module reload with defined state/lifecycle behavior | Validate before transactional replacement |

Implement these as maintained Rust modules and a versioned standard library. Native
plugin compatibility is a separate ABI project; it is not the foundation for these
features. ApX consumes general host APIs and separately negotiated game features
rather than embedding product rules into the compiler or compatibility codec.

Network/database/file work must not block simulation. Host operations return
awaitable results through bounded queues with cancellation/shutdown semantics.
Operators define service roots/endpoints/credentials; scripts receive capabilities
instead of private engine configuration or unrestricted host memory. Credentials
and arbitrary script contents must not enter engine diagnostics.

## Runtime ownership and expansion

Do not reproduce packed 16-bit script IDs, fixed string/hash pools, ten file handles,
or old object/thread/stack arrays as engine limits. Use wide typed handles, growable
intern/object/function/module storage, configurable budgets, and actionable
exhaustion categories. Account for live objects, strings, bytes, stacks, pending I/O,
instructions/time, and host resources separately. Allocation, collection, and
scheduling strategies need measured workloads and an ownership ADR. Do not choose
a collector without assessing language semantics or claim unlimited memory.

Long-running scripts need configurable scheduling/work budgets and per-module
profiling. Compatibility tests decide where old scripts may yield, time out, or
resume. Modern async features must not silently change legacy wait/notify ordering.
Reload needs explicit rules for active frames, closures, handles, subscriptions,
and pending I/O.

Separate frontend/source diagnostics, bytecode validation, runtime/scheduling,
standard library, and host bindings by actual ownership. Create crates when
implemented responsibilities justify them; do not add empty scaffolding. Shared
language/runtime code must remain reusable by a future client. Server-only HTTP/
database/admin capabilities do not become client requirements or implicit protocol
extensions. Features needing client changes require negotiated support.

## Evidence and delivery

Obtain source-grounded legacy syntax, numeric, thread/event, compiler/VM, builtin,
and stock-mod conformance cases. Implement bounded verified execution and integrate
packaged/loose scripts before claiming GSC support. Add modern features in testable
vertical slices with examples, source-map/debugger coverage, malformed-bytecode
checks, cancellation tests, and replayable scheduler behavior. Keep a support/
version matrix.

Current content readers do not compile, execute, profile, reload, or debug GSC.
The dependency-free cod4x-script kernel now implements numeric truth/negation/
complement and explicit-profile wait conversion, checked against 827 original
function cases. It owns no bytecode, objects, threads, or host services. See
[ADR 0012](decisions/0012-script-semantics-boundary.md) for exact scope and limits.
Bounded immutable source/line/span views now supply byte locations for future
frontend/tooling without normalizing input or exposing scripts through Debug.
They are not a parser, compiled source map, or debugger; see ADR 0013.
Bounded raw legacy lexing now retains trivia, exact spans, and original terminal
codes using recovered tables/actions. Its 724 source-derived replay cases are not
original parser execution. This stage does not intern names or execute semantic
actions; see ADR 0014. Modern keywords remain opt-in future work.
Legacy literal decoding now produces owned ordinary/localized byte strings and
checked scalar values without fixed parser scratch storage or string-table IDs.
A 100,000-byte string decodes under explicit budgets. The 545 action/CRT replay
cases do not establish original parser/interner execution; see ADR 0015.
Legacy structural syntax now builds flat script/fragment trees from recovered
grammar data, retaining source terminals without native semantic actions. Ninety-
nine structural replays, exact resource limits, and 12,000 nested groups pass the
focused tests; syntax success does not validate literals, bind names, or execute
GSC. See ADR 0016. This profile does not enable modern keywords implicitly.
Typed top-level module syntax now exposes ordered includes, function signatures,
animation-tree directives, and developer markers without interning or binding.
Synthetic tests retain 66,000 functions and 1,024 ordered parameters, proving only
frontend representation. That stage leaves name/developer policy to binding and
retains structural bodies; see ADR 0017.
Local function binding now validates active name uniqueness and developer-region
rules under an explicit mode, preserving inactive definition metadata. Borrowed
ASCII identifier keys match 125 original helper cases, including native long-name
case fallback without a storage ceiling. This does not bind imports, parameters,
locals, calls, or builtins and does not lower/execute bodies; see ADR 0019.
The scalar kernel now adds numeric pair coercion, six explicit language comparisons,
and float-result division against 12,416 original-helper cases. Legacy tolerance,
unordered comparisons, mixed-number rounding, and signed-zero errors remain
explicit; see ADR 0020. This is not full operators, constant folding, or execution.
Owned plain strings now concatenate under checked result-byte budgets without
the native 8190-byte payload ceiling. Tests distinguish 54 unchanged native results
from 54 deliberate capacity expansions, up to 200,000 bytes. This is a value helper,
not VM integration, aggregate heap accounting, localized/mixed coercion, or a client
wire expansion; see ADR 0021.
The scalar kernel also implements add/subtract/multiply and integer-only bitwise,
signed remainder, and masked shifts under an explicit archival instruction profile.
Tests include 10,701 defined original C cases and 818 isolated instruction cases;
MIN%-1 becomes a controlled error. These helpers are not a compiler or runtime.
See ADR 0022 for evidence scope and intentional invalid-input changes.
Typed borrowed expression-operator views now preserve precedence, operand order,
original spans, prefix operations, and separate short-circuit branches. Primary
descendants await lowering. Classification is constant-cost and performs no
descendant validation/evaluation; complete expressions/IR/bytecode and execution
remain open. See ADR 0023.
Separate typed primary views now preserve all primitive roles, including original
signed-literal tokens, grouped expression lists, unresolved calls/references,
special roots, and animation/debugger syntax. Primary classification alone does
not validate arity/literal values, bind descendants, or enable runtime services.
See ADR 0024; full semantic lowering/compilation/execution remains open.
Ordered expression-list indices now retain complete source operands under explicit
count/two-pass work/logical metadata budgets, including 70,000-entry tests. Native
prepend actions do not become an implicit runtime evaluation policy. List arity,
descendant semantics, executable argument capacity, and VM behavior remain open;
see ADR 0025. This is frontend capacity, not expanded running GSC support.
Named/indirect and function/method call views now expose immediate/thread spelling,
raw callee/module tokens, unresolved receivers, and separately budgeted arguments.
Function-reference views preserve original spelling and wide borrowed names. These
do not resolve modules, enable builtins/host services, evaluate calls, or schedule
threads; source/runtime evaluation order remains separate. See ADR 0027.
Reference views now retain raw identifier/dollar/field/index/reserved-self forms,
including deeply nested objects walked iteratively. The legacy grammar treats
.self and .Self differently. Scope/type/mutability checks, dollar/self runtime
availability, actual values, and reference execution remain open; see ADR 0028.
Owned scalar instruction programs now verify targets, reachable stack heights,
joins, returns, and fallthrough under explicit storage/work quotas. Pure execution
supports undefined returns and numeric operations/branches with separate stack/
fuel admission; failures retain charged work and peaks. Tests route 22,383 existing
source-grounded scalar cases through actual execution and exercise 4,096 hostile
streams, 70,000-index targets, and 4,096-value stacks. This is not a GSC source
compiler/full VM, general undefined/value semantics, heap, scheduler, serialized
bytecode ABI, or host standard library. See ADR 0029; S5 remains open.
Restricted source compilation now accepts zero-parameter empty/single-scalar-return
functions above the typed views/kernel. It emits owned code and original source
maps, preserves short-circuit branches, and validates before publication under
separate compiler/literal/verifier/execution scopes. Unsupported bodies/values,
including unreachable calls, fail explicitly. Twelve tests cover this path,
operator selection/signed boundaries, ownership/privacy, exact limits, deep input,
and branch destinations beyond 16 bits. This does not compile whole stock modules,
bind locals/calls, allocate objects, schedule threads, or enable host services.
See ADR 0030; modern-mode language and standard-library work remains required.
Typed statement views now cover all direct assignment/update, return/wait/call/
event, control-flow/body/label, developer/debugger/profile, and empty-statement
roles. Supplied wrappers/direct operation locations remain available without
walking descendants. Specialized event lists and optional for-condition/update
syntax retain their own profiles. Separate ordered body indices retain 70,000
immediate statements under preflighted count/two-pass work/logical storage budgets,
and failed work remains visible. Nested bodies stay separate and flat-owned.
The scalar compiler reuses typed return handling without broadening its profile.
Local binding, writable references, control-flow legality/lowering, actual events/
waits, developer policy, and general-body execution remain required; see ADR 0031.
Typed scalar execution now owns immutable parameter/local layouts and supports
checked local loads/stores. Parameter arguments copy into fresh initialized slots;
others remain uninitialized, independently of a stored Undefined value. Verification
checks every local operand/count, and execution admits local count/logical bytes
before allocation, separately from operand-stack/fuel limits. Count bounds cover
initialization work; instruction fuel is not an aggregate CPU/RSS guarantee.
Tests retain 70,000 internal arguments/slots without native short/local ceilings.
The exact internal argument contract still needs a source-grounded legacy arity
adapter. Source local binding/definite assignment, call frames, general bodies,
objects, scheduler, and native VM acceptance remain open. The restricted source
compiler keeps its zero-parameter profile; see ADR 0032.
A separate straight-line scalar-local source entry point now binds parameter and
plain local names, assignments/reassignments, empty statements, and final or
fallthrough returns. Initialized reads use legacy name identity and wide slots;
new targets remain uninitialized until RHS lowering/store finishes. Skipped
logical source still undergoes name checks. Code/maps own no source or names, and
this profile publishes after verification without pre-executing made-up arguments.
The result records its profile and optional validation steps; original pure-return
validation remains unchanged. Binding count/identifier-byte visits/logical table
payload share compiler admission; table overhead, body indices, kernel scopes and
RSS remain explicitly distinct. Tests compile/invoke 70,000 locals, retain a
100,000-byte name, and lower 12,000 groups. This profile rejects control flow,
calls/objects/events, nonfinal returns and duplicate
parameters. Full native binding/arity/module acceptance, calls/frames, richer
values, scheduling, builtins and modern extensions remain required; see ADR 0033.
Plain-name compound assignments now cover the ten guarded legacy spellings through
existing scalar numeric instructions. Targets must already be initialized; prior
target and RHS alias values are preserved before the checked store. Original
compound-token maps and existing name/task/code budgets remain in use. Numeric
failures stay runtime errors in this source profile. Richer overloads, object/index
targets, side-effecting RHS order, postfix updates and full statement execution
remain separate work; see ADR 0034.
The local profile additionally supports standalone integer-only postfix statements
for initialized plain names. A dedicated unary instruction rejects floats and
uses established wrapping integer arithmetic; stores, source update-token maps,
name/task/code budgets and caller isolation remain shared. Source guards now pin
the VM assembly and connect grammar/compiler tags/opcodes with integer type checks
and 32-bit word updates. These do not execute original handlers or establish full
native variable-reference/unwinding/VM behavior. Postfix expressions, object/index
updates, modern numeric rules and general control flow remain open; see ADR 0035.
Scalar script invocation additionally exposes a guarded legacy argument profile:
already parameter-ordered supplied values fill a prefix, missing parameters become
initialized Undefined, and extras fail. Nonparameter locals remain uninitialized.
The admitted local vector fills directly without a temporary padded argument
array; count/byte/stack/fuel quotas and caller isolation remain shared. Existing
exact/empty internal entry points retain their own contracts. Source-compiled
scalar parameter functions use this profile in tests, but function-call linking,
builtin validation, native evaluation/push order, receivers/frames/scheduling and
whole error-unwinding/VM compatibility remain open; see ADR 0036.
The scalar-local profile now also lowers ordinary blocks and if/if-else through
flat tasks and checked forward relocations. Function-wide slots retain stable wide
indices. Initialization snapshots restore incoming facts before else and merge
only branches that reach the join; a returning branch does not constrain a
surviving path. Both syntactic branches are checked even for constant conditions.
Return remains final in its immediate body, and later structurally unreachable
statements fail. Snapshot flag storage shares compiler metadata plus a separate
live-byte quota; flag initialization/copy/restore/merge visits share global work.
Tests cover exact/one-less limits, 70,000 local snapshots, long forward jumps,
and 12,000 nested branches with successful and failed flat cleanup. This restricted
policy does not prove native scope/constant-folding/diagnostic order or whole
compiler execution. Loops, switch, developer blocks and general values/calls remain
open; see ADR 0037. Pure-return compilation keeps its narrower validation contract.
Scalar while statements now reevaluate numeric/lazy conditions through existing
verified instructions and backward jumps under execution fuel. Flat body stages
use existing task/work/metadata scopes; incoming initialization facts are restored
at the exit because zero iterations remain possible in this conservative source
policy. Body-only writes do not authorize later reads, and both condition/body
syntax is checked without constant folding. Nine tests cover real loop results,
returns, condition/operator maps, fuel exhaustion/exact admission, compiler quota
boundaries, 70,000 local snapshots with a long backward jump, and 12,000 nested
loops with bounded failed cleanup. The combined large fixture explicitly admits
two million syntax nodes in its separate frontend scope; production defaults
remain unchanged. Source guards tie selected compiler/header/VM branch/backward
roles to the pinned source without executing original routines. Native loop
watchdog timing, scopes/constant folding/error-order parity, for/break/continue,
switch, richer values/calls and scheduling remain open; see ADR 0038.
Scalar while now additionally supports scoped break/continue. The nearest active
context is saved/restored in already admitted flat tasks; both transfers terminate
their current structural path and stay excluded from subsequent conditional joins.
Break fixups form a monotonic single-use chain inside admitted pending instruction
records, avoiding a second allocation/array. Each bound link consumes global work;
unresolved links cannot enter a verified program. Continue targets the actual
condition head and shares instruction fuel. Six tests cover nested targets, joins,
illegal/unreachable source, maps/fuel/caller isolation, exact compiler limits,
20,000 fixups with wide exit destinations and 12,000 nested contexts including
failed cleanup with saved pending outer fixups. Source guards reject 38 in-memory
compiler-profile mutations without executing original routines. For/switch contexts,
native block cleanup/scope/watchdog/diagnostic order, richer values/calls and full
GSC acceptance remain open; see ADR 0039.
Scalar for now lowers initializer/condition/body/update/backward phases through
flat tasks. All optional clause combinations are supported; clauses accept only
scalar assignment/compound/postfix/empty forms. Deferred continue fixups reach the
actual update, with admitted snapshots merging every continuing body path and
excluding break/return. An unreachable syntactic update still checks incoming
facts; exits conservatively allow zero iterations even for absent/constant true
conditions. This restricted policy may reject body-only names in unreachable
updates and does not establish native scope/constant-folding/error-order parity.
Generalized loop contexts and instruction-record transfer chains retain shared
metadata/work/fuel admission. Nine tests cover phases/joins/returns/errors and
maps, all eight optional combinations, mixed loops, exact limits, 20,000 deferred
continues with wide destinations and 12,000 nested for contexts/failed cleanup.
Selected grammar/compiler phase profiles reject 35 in-memory mutations without
executing original routines. Switch, richer values/calls/scheduling and full GSC
acceptance remain open; see ADR 0040.
Owned legacy vector helper values now represent three binary32 coordinates with
explicit numeric broadcast/coerced-pair/operator/error APIs. Multiplication and
division are component-wise. Addition retains the separate string-aware coercion
policy and rejects mixed vector/numeric operands. Equality compares coordinates
exactly: signed zeros and like infinities compare equal; NaNs do not. Scalar float
tolerance does not apply. A zero in any denominator coordinate returns a typed
error without exposing native cleared intermediate output. Inline values allocate
no vector pool and Debug/errors redact data; they do not establish VM heap/RSS,
reference cleanup or error unwinding. Scalar kernel/source profiles are unchanged.
Fourteen original construction/coercion/operator/math functions plus original
math macros run under bounded stubbed services at O0/O2. All 19,852 recorded cases
match: 756 constructions, 1,736 coerced pairs, 6,944 arithmetic outcomes and 10,416
comparison outcomes. These include 8,272 intercepted type/division error outcomes,
not native error-unwinding successes. Construction/coercion bits and finite math
bits match; arithmetic NaNs compare by class under the nearest/SSE FTZ/DAZ-off
profile. Native source-component evaluation/push order, constructor error cleanup,
kernel/source vectors, richer heap values and full GSC remain open; see ADR 0041.
The owned primitive kernel now additionally represents inline vectors and verifies
MakeVector's three-input/one-output stack effect. Its explicit bottom-to-top z/y/x
contract is an internal typed instruction profile, not native/source push-order
evidence. Shared numeric/vector dispatch preserves scalar behavior and vector
rules, with fixed nested errors; unsupported truth/integer/component types fail.
Constants, duplicate, locals, arguments and returns copy inline data. Existing
size_of-based code/stack/Option-local limits include larger payloads before
allocation, and instruction fuel remains shared. Five added kernel tests replay
18,116 original construction/arithmetic/comparison outcomes, test invalid types,
underflow, exact/one-less storage/fuel, missing/default arguments, privacy and
70,000 vector slots. Coercion-only 1,736 rows remain helper evidence. Both hostile
4,096-stream corpora now include vectors/MakeVector; local streams additionally
invoke vector arguments. Two source tests pass vector arguments through existing
restricted syntax and retain operator maps/caller isolation. Source vector literals
remain unsupported; native construction error indices/cleanup, source component
evaluation/push order, heap/calls/scheduling and full GSC remain open. See ADR 0042.
The compiler/VM, expanded standard library, and language features remain required
implementation work. Roadmap S5 requires named stock modes and existing mods to
execute correctly; modern-mode features need their own acceptance evidence.

## Remaining legacy acceptance workstreams

The current planning estimate is roughly 10–12 major workstreams, not a count of
small coding tasks or a delivery promise. This working breakdown uses twelve;
several interact and each requires multiple implementation/validation changes.
Partial scalar/source support does not close any full legacy acceptance gate.

| Workstream | Required completion evidence |
| --- | --- |
| Full semantic frontend | Every supported legacy statement/expression/directive lowers with valid diagnostics and budgets |
| Binding and control flow | Locals/scopes/initialization, branches/loops/switch, returns/break/continue match the named source profile |
| Complete values and heap | Strings/vectors/arrays/objects/references, identity/lifetimes and operator overloads work under admitted memory |
| General code generation | Full expression/statement lowering preserves observable evaluation/control flow and source locations |
| Modules and linking | Script lookup/imports/functions/developer/animation references resolve and publish atomically |
| Calls and frames | Functions/methods/indirect calls, native argument rules and unwinding work without native frame/slot ceilings |
| Cooperative scheduling | Threads/waits/frame waits/notify/waittill/match/endon obey game-time and cancellation semantics |
| Legacy builtin library | Required builtin names, arguments/results/errors and service behavior match supported mods |
| Engine/game integration | Real entity/player/asset/collision/HUD/game services support the scripts, not fabricated host responses |
| Errors and developer tools | Runtime failures cleanly unwind; diagnostics/debugger/profiling and lifecycle behavior have explicit contracts |
| Compatibility validation | Named stock modes/mods run with recorded content/version evidence on target platforms |
| Resource and operational validation | Fuzz/load/soak/reload, memory growth, exhaustion/cancellation and performance gates pass |

Some engine integration depends on the server's asset/simulation milestones. A
compiler that emits code without those services is not full existing-mod support.
Expanded language, asynchronous services, modern standard library and their
tooling/capability contracts add further work beyond this legacy acceptance list.
