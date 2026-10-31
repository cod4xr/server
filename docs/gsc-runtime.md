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
The compiler/VM, expanded standard library, and language features remain required
implementation work. Roadmap S5 requires named stock modes and existing mods to
execute correctly; modern-mode features need their own acceptance evidence.
