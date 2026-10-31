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
forms explicitly await lowering. Classification is constant-cost and performs no
descendant validation/evaluation; complete expressions/IR/bytecode and execution
remain open. See ADR 0023.
The compiler/VM, expanded standard library, and language features remain required
implementation work. Roadmap S5 requires named stock modes and existing mods to
execute correctly; modern-mode features need their own acceptance evidence.
