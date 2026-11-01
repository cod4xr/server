# Expanded GSC runtime contract

ADR 0077 adds closed supplied-file compilation into one owned verified image,
including qualified calls/references, local namespaces, developer rules and shared
compiler/runtime/heap admission. The script_package example runs two real sources;
wide/deep/ownership/resource tests and 38 native source-role mutations scope the
evidence. Caller source-loading phases must reject or resolve directives separately.
Filesystem loading/includes, source arrays, scheduling and stock/mod parity remain open.

ADR 0076 adds budgeted canonical virtual script identities and owned names for
future file-qualified lookup. Native/model/resource/wide/privacy tests cover naming;
module catalogs and cross-file calls/references follow ADR 0077; includes/loading remain open.

ADR 0075 establishes the next ownership requirement: native acyclic holder release
removes array aliases before a later COW decision; cycles retain them. The 210-trace
helper corpus/model does not implement this in nested/source execution. Ordinary
object frame/field owners and array owners need a combined logical-lifetime policy
separate from deferred payload collection before source arrays activate.

ADR 0074 adds staged populated copying for current primitive/shared-struct entries,
preserving legacy key reversal and shared identities with complete admission before
publication. Wide/churn/model/resource/native-state tests cover the component.
Nested arrays, cycle-alias markers and VM/frame/field/scheduler ownership still
gate source activation; this does not close the complete GSC workstream.

ADR 0073 adds typed unique-value array entries using existing metered fields/index.
Reads/enumeration borrow nonowning snapshots; writes/deletes require one owner and
retain raw Undefined until explicit deletion. Wide/churn/model/resource/native-state
tests assess storage, not source array execution. Nested COW, nested
array values, cycle-alias markers and VM ownership still gate source activation.

ADR 0072 implements the managed empty-data portion of proposal 0071: explicit
host-owned locations, nonowning snapshots and empty detachment under wide quotas.
Tests retain 70,000 arrays/owners and churn 100,000 times within two data/value
slots; an independent model checks 16,384 mixed operations. Source arrays remain
rejected until indexed/nested COW, cycle-alias markers and frame/field/scheduler
ownership are implemented. No complete GSC workstream closes from this component.

ADR 0070 implements opt-in budgeted reference-cycle boundary analysis, validated
against native trace topologies, 4,096 independent graph models and wide/churn/
resource workloads. Struct-only collection is unchanged; ADR 0072 adds managed
empty-array roots. Alias markers, populated COW and source arrays remain open.

Array ownership assessment now includes 20 selected native cycle-alias traces:
root release can retain aliases that still trigger COW/key-order reversal. A
collector must preserve that observable behavior when reclaiming cycle payload.
The proposed logical-marker approach remains under assessment (ADR 0060), and
arrays remain unavailable to source execution.

Ordinary struct equality/inequality now validate and compare shared identities
(ADR 0069), with fixed logical work and no field scans. Native ordinary-pointer
helper evidence and Rust wide/reference/source/process tests cover the subset;
array comparison, ordered/mixed coercion and full object semantics remain open.

Ordinary bound-source struct reads/simple writes now use independent immutable
field-name tables, preserving aliases, missing Undefined, Undefined deletion and
RHS-before-receiver order (ADR 0067). Wide/cyclic/deep/resource/source/process
tests cover the subset. ADR 0068 adds compound/postfix field updates with guarded
distinct receiver evaluation orders and retained old values through RHS collection.
Arrays, entity/method/self behavior,
scheduling and full stock/mod acceptance remain open.

Bound-source spawnstruct now allocates empty shared structs through an explicit
object context (ADR 0066), retaining evaluated operands during pressure collection.
The offline runner owns both stores and exposes only opaque object output/numeric
metrics. Array COW, methods, entity lifetimes and scheduling remain
open; this does not close a complete GSC acceptance workstream.

Shared struct storage now has explicit-root iterative graph collection and composite
string roots (ADR 0063), with wide/churn/cycle/resource/model workloads. This is a
component boundary. ADR 0064 adds host reference transport through an explicit
object-aware context, complete live frame/caller/cache/external roots and separately
accounted object/string pressure collection. Pure/string-only invocations still
reject object snapshots. General object semantics
remain open. Legacy arrays require their distinct copy-on-write, nested-copy and
key-order behavior (ADR 0060).

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
Compiler APIs now use function terminology: CompiledFunction, FunctionCompilation
accounting/budgets/resources, FunctionCompileError/Kind and FunctionSourceProfile.
compile_scalar_return_function selects PureScalarReturn; compile_primitive_function
selects PrimitiveStatements. Compiler leaf modules/private types and the shared
test helper use matching names. This is an unreleased Rust API cleanup with no
syntax/runtime/quota/proof/source-map change or additional profile/implementation.
Diagnostic labels follow the new names; historical ADRs retain earlier naming.
No full GSC/compiler/module acceptance follows from this cleanup; see ADR 0043.
PrimitiveStatements now accepts source vector triples alongside singleton grouping.
The builder owns one explicit profile used for permission, validation and result
tagging; PureScalarReturn retains its narrower syntax. A focused group leaf probes
only the exact bounded list shape, charges visits globally, and queues flat tasks
to evaluate z/y/x before MakeVector. Construction maps to the whole original group;
component/operator failures keep their own spans. Empty/two/four-or-more groups
and unsupported skipped source fail explicitly. Primitive compilation still does
not pre-execute invented arguments or emulate native constant folding/error order.
Eight added tests cover real source vector values/operations/loops, float bits,
rightmost failing component and initialization order, maps/privacy/profile guards,
exact compiler and separate literal/proof/runtime limits, 70,000 vector locals and
12,000-deep vector expressions with bounded failed cleanup. The wide fixture
explicitly admits 3M syntax nodes/4M edges/512MiB frontend metadata; production
defaults stay unchanged. The verified parse-tree helper and selected grammar/
compiler/VM/constructor roles now have source guards rejecting 33 in-memory
mutations. These do not execute native parser/compiler/VM routines. Full component
side effects/constant folding/error indices/ref cleanup, vector indexing, richer
values/calls/scheduling and GSC/mod acceptance remain open; see ADR 0044.
Legacy vector reads now expose strict integer indexing with copied float results
and fixed type/range errors that retain neither coordinates nor attempted indices.
The primitive kernel's Index consumes bottom-to-top index/object under existing
proof/byte/fuel admission; other objects are not yet indexable. Primitive source
reference reads schedule index, then object, retain bracket/descendant spans and
initialized-name/identifier budgets. PureScalarReturn and indexed-write rejection
remain explicit. Original Scr_EvalArray produces 299 O0/O2 outcomes: 39 successful
exact-bit reads and 260 intercepted errors. All run through verified programs;
the numeric-only helper API covers 286 rows, excluding 13 vector-index type cases.
Shared oracle source building preserves the prior 19,852 vector fixture bytes.
Eight added tests cover that corpus, stack/type/fuel/byte limits, caller/default
isolation, source loops/reconstruction/maps/order/profile/write rejection, exact
compiler admission and 12,000-deep reference reads/failed cleanup. The wide vector
source fixture also reconstructs coordinates from wide local slots. Both hostile
corpora include Index. Selected parser/compiler/VM/helper/read-only roles reject
27 mutations; these do not execute original parser/compiler/VM or native service/
error-cleanup code. Arrays/strings/heap, calls/scheduling/host services and full
GSC/mod acceptance remain open; see ADR 0045.
Primitive comparisons now allow equality/not-equality of two Undefined values;
all mixed Undefined pairs and Undefined ordering fail with fixed UnmatchingTypes.
The focused comparison dispatch preserves numeric tolerance and vector equality
without using derived Rust data equality. Four added tests replay 402 actual
O0/O2 C-helper outcomes (two successes/400 errors) through verified instructions
and cover both source profiles, lazy branches, missing/stored Undefined, rejected
uninitialized locals, operator locations and error privacy. Existing instruction,
proof, byte/local/fuel budgets remain in force; there is no new allocation or
value category. This is not object/string/builtin/native cleanup/full-VM parity;
see ADR 0046.
VerifiedKernelModule now admits a complete primitive function inventory with
forward/recursive direct Call targets, complete target checks (including unreachable
code), owned copied proof and per-function stack/control-flow verification.
All code/table/current-verifier payload and verification work share global budgets.
The standalone verifier rejects unresolved calls. The iterative executor shares
frame stepping with standalone execution, preserves private copied arguments and
missing Undefined suffixes, and releases each frame on return. Global instruction
fuel, separate conservative setup work and aggregate reserved frame/operand/local
count/byte admission prevent per-call budget resets. Fixed errors preserve
function/instruction, global work and peak admitted counts without value contents.
Fourteen added typed-runtime tests cover calls/defaults/returns/privacy/recovery,
exact/one-less global limits, repeated shallow calls, 12,001 frames, 66,000 functions
70,000-argument calls/missing suffixes and 4,096 hostile inventories. This is not native call ABI, argument evaluation/
push order, source arity diagnostics, ref/error cleanup or original VM execution.
Import linking, methods, heap values, scheduling and
full GSC/mod acceptance remain open; see ADR 0047.
Excess call arguments now fail at invocation, after caller argument instructions
and before callee allocation/body. Unselected paths do not fail, while every
reachable call still needs its declared operand count under proof. The check uses
parameters rather than total local slots. Three added tests cover skipped and
selected counts, caller/body error precedence, argument-fitting-local confusion,
privacy/recovery and huge reachable count underflow. Hostile inventories now
include dynamically failing excess counts. Selected locked compiler/VM argument
roles and mutation guards supply evidence without executing the native routines;
source/native cleanup/error-index/full-VM parity stays open. See ADR 0048, which
supersedes ADR 0047's static excess rejection.
Bound primitive source compilation now connects local validated symbol indices to
owned executable modules. Named immediate calls resolve against one complete shared
active inventory before body lowering; forward/mutual/recursive expression and
statement calls retain rightmost argument evaluation and runtime excess checks.
Existing isolated profiles stay narrower. Caller visibility is distinct from
developer inclusion: ordinary references to developer-only targets fail, disabled
unsupported bodies remain metadata, and empty/all-disabled modules fabricate no
entry. Code/maps/layouts stage privately and the complete kernel inventory verifies
before publication, preserving original declaration-to-active-function mapping and
function/instruction spans after source/tree/symbol ownership drops.
Global declaration/function/compiler-work/instruction/logical metadata budgets
include scope/records/staging/current lowering overlap; individual function limits
remain separately enforced and clamped to remaining global admission. Body/literal/
source/CST/symbol/kernel scopes and allocator/hash/spare-capacity overhead stay
explicitly outside that counter; it is not RSS. Fifteen source tests exercise real
calls, branches/loops/vector results, error/name order, arity/maps/privacy/ownership,
case identity including native long-name fallback, developer modes/unsupported
forms, exact/one-less scopes, 70,000 arguments/parameters, 66,000 functions and
12,000 nested calls with flat failed cleanup. Selected direct/deferred developer
reference guards reject 11 mutations; argument timing guards reject 38, while 99
structural fixture replays stay unchanged. These are source-role guards plus Rust
execution tests, not original compiler/VM/linker execution. Imports/animation
resolution, strings/arrays/objects/receivers, builtins/threads/waits/
host services, full native diagnostics/cleanup/mod and Linux gates remain open.
See ADR 0049. Its empty-inventory decision supersedes ADR 0047; empty individual
instruction streams still fail proof.
Bound source now supports local function-reference values and immediate indirect
calls. One shared resolver preserves case/developer policy for named calls and
references. Flat lowering evaluates rightmost arguments before the target expression;
copied references pass through parameters/locals/returns without heap allocation.
Each completed immutable kernel module mints a checked nonreused opaque identity.
References compare by identity, reject mixed/ordered comparisons, and invoke only
within their owning module. Foreign/dropped owners cannot resolve in a replacement.
Verifier checks static targets and argument-plus-target arithmetic even in skipped
code. Execution shares existing iterative frames, arity, fuel/setup/live-payload
limits; byte formulas use actual value/instruction layout. Tests cover parallel
identity uniqueness/full-domain exhaustion, foreign/drop/redaction, factories,
higher-order recursion, evaluation/error order, source restrictions and quotas,
all exact/one-less runtime limits, 66,000 targets/70,000 arguments, 12,000 nested
source calls and 4,096 hostile inventories. Actual C O0/O2 comparisons yield 2,664
cases (72 successful/2,592 errors), using synthetic labels mapped bijectively to
valid Rust references; native code pointers/allocation/invocation are not tested.
Selected compiler/header/VM roles reject 31 mutations alongside existing argument,
grammar and visibility guards. These do not execute the native compiler or VM.
See ADR 0050. Self/receivers, imports/reload, closures, heap values, scheduling,
builtins/game services, full legacy diagnostics/cleanup/mod and Linux remain open.
Four pure builtin calls now work in bound primitive source:
isint, isfloat, isvector and isstring. Active script definitions shadow them, preserving
canonical identifiers and developer restrictions; absent names resolve only to
this sealed subset. No plugin registry, host callback or ambient authority is
introduced. Builtin references/external/receiver/thread forms remain rejected.
Arguments evaluate before exact-one runtime arity checks, including skipped-call
and argument-error timing; proof still admits every reachable operand. Pure
instructions share global fuel and actual-size stack/code admission without an
argument array, extra frame or inherited 255-argument cap. New C files were hashed
against their pinned Git blobs. Actual extracted functions produce 832 O0/O2
count/tag outcomes: 224 run through verified Rust instructions (32 successes/192
arity failures), while 608 remain explicit helper-only evidence for unavailable
native tags and UINT32_MAX declared counts. This does not allocate native values
or execute the original VM/compiler/error unwinder. Selected registration/type/
arity/lookup/developer/argument guards reject 46 mutations; 99 structural replays
stay unchanged. Tests cover source order/arity/shadowing/modes/maps/privacy/quotas,
256 source and 70,000 typed arguments, exact/one-less proof/execution resources,
12,000 nested pure calls with one frame, malformed proof and hostile streams.
See ADR 0051. General builtins, heap/entity lifetime/isdefined, scheduling/game
services, full diagnostics/mod and Linux acceptance remain open. Further compiler,
standard-library, language and engine integration remain required
implementation work. Roadmap S5 requires named stock modes and existing mods to
execute correctly; modern-mode features need their own acceptance evidence.

PrimitiveStatements and bound source now lower integer switch, case and default
with once-only key evaluation, strict integer typing, adjacent labels, source-order
fallthrough, default anywhere and nearest-break ownership separate from continue.
Exit initialization includes actual break/fallthrough paths and an unmatched path
only without default; returning/continuing paths do not establish after-switch
facts. Existing nonfinal-return/unreachable-source restrictions remain explicit.
Case records share control/metadata budgets with snapshots; logical sorting/tree-
search/emission/fixup/fact work is charged. Flat saved contexts release all payload
before publication. Balanced typed dispatch has checked fixed footprints and
shrinking child ranges; it shares global fuel without a linear case scan or new
serialized/native ABI. Full int32 keys intentionally expand native encoded-ID
bounds, while wide case counts avoid the native short. These are capacity changes,
not native invalid-input parity. Tests reach 66,000 cases with at most 195 executed
instructions per tested lookup, and 12,000 nested switches with flat failed cleanup.
Source tests cover nesting, for-update continues/fuel, joins/excluded paths, defaults/
fallthrough/duplicates/types/maps/privacy/isolated profiles and exact-one-less quotas;
typed tests preserve complete word boundaries and reject other primitive categories.
Selected native scope/type/sort/duplicate/domain roles reject 39 mutations; 99 syntax
replays stay unchanged. These do not execute native switch routines. See ADR 0052;
string switch, full heap/native scope/diagnostics/cleanup/mod and Linux gates remain
open. No complete legacy workstream is closed by this subset.

The CLI now exposes --execute-gsc PATH for this implemented primitive subset.
It reuses bounded source loading/binding, rejects unresolved include/animation-tree
directives, compiles every active body and invokes canonical active main through
owned declaration mapping. It supplies no explicit arguments, preserving initialized
Undefined for missing parameters. Default compiler/proof/global execution limits
remain mandatory. Source/compiler/runtime failures are located and sanitized;
stdout alone carries typed result data, while logs retain category/work/peak usage.
Function results expose no identity. The mode opens no listener and permits no
unknown host services. Real process tests cover mappings/developer settings, all
primitive outputs, directives/missing entries/active unsupported bodies, compiler/
runtime/fuel failures, source locations/privacy, JSON/human logs and offline options.
Write/flush errors propagate. The runnable primitive_demo.gsc example produces
integer 18 under the normal target build. See ADR 0053; this is not full-mod,
heap/builtin/scheduling/game-service or joinable-server acceptance.

## Remaining legacy acceptance workstreams

An immutable string-store component now owns exact bytes behind wide checked
references, with separately admitted live/payload/retained metadata/work/scratch
and atomic failed complete-root reclamation. Tests cover 70,000 live values,
100,000 temporary values under 64 slots, hostile operations, namespace/generation
exhaustion and native plain-string helper results versus capacity expansion.
The reproducible string_heap_workload example records logical usage/work and
elapsed time; it does not measure RSS or full VM performance. ADR 0054 accepts
this component only. ADR 0055 now adds explicit heap-backed typed execution and
supplied-string source arguments with plain addition/equality. Pressure collection
preserves current/caller frames and external roots under shared work/scratch limits.
Tests include 12,000 live frames, 100,000 loop allocations using four slots, failed
work/fuel/lifetimes, 4,096 hostile inventories and real source maps. ADR 0056 now
adds owned typed literal tables and invocation caches with complete proof/work/
scratch admission, 70,000-row and repeated-large-value tests. ADR 0057 now adds
bound plain-source literals and offline escaped output, with source-level wide/
large/pressure/resource/map tests and process privacy/output checks. ADR 0059 now
adds distinct localized values, same-type equality and raw escaped output, with
3,024 actual C comparison/arithmetic outcomes, complete predicate tag replays and
mixed 70,000-row/source/rooting/resource tests. ADR 0061 now adds plain-string byte
indexing/size with 896 native outcomes, source/process/rooting/resource tests and
45 selected-role mutations. Translation/mixed coercion, string switches,
objects/cycles/scheduling and general library/game roots remain integration work.

ADR 0062 adds isdefined for the implemented primitive categories, with a required
first argument and evaluated extras. Optional script parameters retain Undefined;
local/developer resolution still precedes fallback. Source/resource/process tests
and isolated archival comparison evidence cover this subset, without claiming
entity/thread lifetimes or the original native builtin/VM/error service execution.

The current planning estimate is roughly 10–12 major workstreams, not a count of
small coding tasks or a delivery promise. This working breakdown uses twelve;
several interact and each requires multiple implementation/validation changes.
Partial scalar/source support does not close any full legacy acceptance gate.

Array ownership research now records 306 C helper traces at O0/O2 under
explicit modeled pool/reference services. These cover key domains/order, recursive
array copies, shared structs, local-slot copy-on-write/autovivification and
clear/set/remove helpers and selected native unlink/free paths. Allocation/hash/
ref/output services and VM call sequences are modeled; collision relocation and
allocator reuse stay open. 69 copy/key and 141 write/grammar/compiler/VM role
mutations reject. Literal and
computed Undefined deletion have distinct parent/temporary-key setup paths.
Rust array/object storage and VM execution remain
unimplemented. The ownership/storage acceptance assessment is ADR 0060.

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
