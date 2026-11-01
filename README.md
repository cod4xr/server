# cod4x-rust

A standalone Rust rewrite of the CoD4X dedicated server, designed to share
protocol and simulation code with a future custom client. Existing CoD4/CoD4X
clients, maps, and mods are compatibility targets. ApX-specific behavior will
live above the reusable engine and require explicit extension negotiation.
Inherited engine limits are removal targets: internal storage uses configurable
resource budgets, while old client/wire bounds stay inside compatibility profiles.
The engine must preserve full content catalogs rather than require asset cuts.

**Status: protocol/content foundation and runnable development probes.**
There is no gameplay, authentication, decoded map loading, complete GSC VM, renderer, or
master-server integration yet. Real local UDP discovery and structured process
logs are tested; interoperability with existing game clients is still unverified.
The complete server rewrite remains open.

## Implemented

- Allocation-free, safe Rust message reader and writer, with CoD4X's separate
  byte and bit cursors, little-endian signed integers, and binary32 values.
- Split-buffer reads, a 128 KiB logical message limit, typed errors, and atomic
  failure without partial writes, output mutation, or cursor advancement.
- 64 reference packets generated from the pinned upstream C functions, covering
  6,144 mixed operations and decoding at every possible storage split boundary.
- Tests for invalid widths, truncation, dirty storage, buffer reuse, special
  floating-point values, cached bits, and exact resource limits.
- Workspace standards and dependency boundaries, a 400-line handwritten source
  ceiling, a pinned Rust toolchain, and Windows/Linux CI definitions.
- Borrowed connectionless/channel envelopes in both directions, explicit special
  transport markers, command tokens, bounded info strings, and ordered legacy
  entity/client index codecs. Another 168 fixtures come from original functions.
- Socket-free channel sequencing and fragmentation in `cod4x-transport`, with
  cancellation, bounded reassembly, duplicate/reordering tests, and reference
  packetization. Wider engine IDs remain separate from legacy peer namespaces.
- Static Huffman compression/decompression verified against 257 codebook entries
  and 32 original C samples, preserving padding behavior without out-of-bounds reads.
- Keyed legacy player-command decoding, checked against 512 original C cases at
  every storage split, all truncation boundaries, and every signed-short melee
  angle. Commands remain separate from simulation and wider engine identifiers.
- Borrowed connect-userinfo parsing with typed critical fields, signed/unsigned
  qport spelling compatibility, credential-redacted Debug output, and explicit
  rejection of malformed or duplicate critical fields. Admission remains open.
- A query-only UDP application with deterministic discovery services, bounded
  per-IP/global reply budgets, shutdown signals, and real process/socket tests.
- Nonblocking development UDP readiness polling with per-datagram lifecycle
  checks, bounded waits, repeated cookie-exchange and queued-burst process tests.
- Structured JSON logs, readable console option, daily file rotation/retention,
  bounded asynchronous output, dropped-record counters, lifecycle events, and
  aggregate traffic metrics. Log regression tests exclude remote payloads.
- Growable encoded asset storage with configurable budgets, exact byte names,
  wide generation-checked handles, unload/reload, and usage accounting. Synthetic
  tests retain 5,000 images and 1,500 models without legacy pool ceilings; format
  decoding and client-visible capacity negotiation remain separate work.
- Read-only IWD content with path/header/metadata budgets, stored/DEFLATE reads,
  CRC and size validation, original override order, and seeded legacy checksums.
  ZIP64 tests admit 66,000 entries; this does not establish old-client ZIP64 support.
- Borrowed IBSP container validation with wide checked offsets, configurable
  budgets, raw unknown/duplicate kinds, independent map instances, and 280 original
  C lookup/checksum cases. Geometry, collision, and fastfile graphs remain open.
- Unsigned IW3 v5 fastfile envelope decoding with configurable payload/block/
  external budgets, exact size and zlib checksum/end checks, fallible output growth,
  and fifteen independent synthetic envelopes. Asset graphs remain unvalidated.
- Borrowed IW3 root metadata with separate null/inline/unresolved script slots,
  preserved unknown asset records, virtual-block accounting, and configurable
  budgets. Synthetic tables retain 70,000 slots/records; graph bodies remain open.
- Typed raw-file-only collections with validated buffers, shared/suffix names,
  root/inserted aliases, scoped temporary-memory accounting, and configurable
  budgets. Mixed stock zones, registry publication, and GSC execution remain open.
- Dependency-free legacy script scalar/wait semantics checked against 827 original
  function cases. This is a tested kernel; the full compiler/VM and expanded language
  features described in the GSC contract remain unimplemented.
- Bounded immutable script-source indexing and borrowed exact line/span views,
  with wide byte locations, explicit LF/CRLF behavior, and redacted diagnostics.
- Bounded borrowed legacy lexemes/trivia from recovered source tables, with 724
  source-derived replays and hostile-byte tests. Original parser parity remains
  unverified; lexing does not itself compile or execute source.
- Bounded decoded legacy literals with owned byte strings, localized-string
  distinction, checked numbers, and 545 action/CRT replays. A 100,000-byte string
  decodes without the old scratch-buffer ceiling; interning/VM capacity is separate.
- Bounded flat legacy script/fragment syntax trees with 99 structural grammar
  replays, original terminal spans, explicit budgets, and 12,000-level nesting.
  This stage runs no semantic actions; general compilation/execution remain open.
- Typed legacy module declarations with ordered includes, functions, parameters,
  animation-tree directives, and developer markers under explicit budgets. Raw
  spelling/duplicates remain preserved; body lowering and full semantic binding remain open.
- Local function binding with original ASCII name identity, source-offset
  duplicate diagnostics, explicit developer-script modes, retained inactive
  definitions, and configurable index budgets. Calls/imports/bodies remain unresolved.
- Legacy scalar coercion, six language comparisons, and float-result division
  checked against 12,416 original-helper cases. Full operators/values/VM remain open.
- Owned plain-string concatenation with checked, fallible, budgeted growth. Tests
  preserve 54 native byte results and deliberately expand 54 native capacity-error
  cases, up to 200,000 bytes. This helper does not implement VM strings or coercion.
- Scalar addition/subtraction/multiplication, integer bitwise operators, signed
  remainder, and explicit x86-profile shifts. Another 10,701 defined original C
  cases and 818 isolated instruction cases cover these helpers; full GSC execution stays open.
- Allocation-free typed expression-operator views preserving source operands,
  precedence, and separate short-circuit branches. Descendants remain unresolved;
  this does not validate literals, bind descendants, compile, or evaluate scripts.
- Separate borrowed views for all 22 primary productions, retaining raw literal
  polarity, grouped lists, call/reference subtrees, object roots, size/array/
  animation/debugger syntax. Local roles do not establish value/runtime support.
- Fallible source-ordered expression-list indices under count/work/metadata budgets,
  tested with 70,000 entries. Runtime call/vector arity and argument evaluation stay open.
- Borrowed named/indirect function and method calls, immediate/thread spelling,
  original module/function references, and separately admitted argument indices.
  These views do not link functions, evaluate receivers/arguments, or create threads.
- Borrowed name/dollar/field/index/reserved-self reference views, preserving nested
  operands and original spelling. Scopes, object types, mutability, and values stay unresolved.
- Owned verified scalar instruction execution with checked control-flow/stack
  joins, separately budgeted fuel/stack, and work-accounted failures. Tests execute
  22,383 source-grounded scalar cases and 4,096 hostile streams; full source compilation,
  objects, calls, threads, host services, and complete GSC execution remain open.
- Restricted source compilation for zero-parameter empty/single-scalar-return
  functions, with owned verified code/source maps, iterative lowering, numeric
  operators, lazy logical branches, and separate compiler/kernel budgets. This
  explicitly rejects general bodies/values and does not activate modules or run stock mods.
- Borrowed typed statements for assignment/update, return/wait/call/events,
  blocks/conditionals/loops/switch labels, developer/debugger/profile syntax,
  and empty statements. Ordered immediate body indices admit both passes/count/
  storage before allocation; local binding and general-body execution remain open.
- Scalar-kernel local loads/stores with immutable parameter/local layouts, exact
  internal argument binding, uninitialized-read errors, isolated invocations, and
  local count/byte budgets. General source binding, native GSC arity, and call frames remain open.
- A separate scalar-local source profile now binds parameters/local names,
  ordinary assignments/reassignments, empty statements and final/fallthrough returns.
  It preserves initialization checks and owns verified code/maps without fabricated
  argument execution. Tests compile/invoke 70,000 locals; remaining general control flow,
  calls/objects/threads, native arity and full GSC/mod acceptance remain open.
- All ten scalar plain-name compound assignments reuse the checked numeric
  operations and source maps. Targets must already be initialized; object/index
  assignment semantics remain separate work.
- Integer-only plain-name postfix increment/decrement, with guarded native
  grammar/compiler/VM type and word-operation profiles. Float updates fail;
  overflow wraps and runtime errors map to the original update token.
- Explicit scalar legacy script-argument invocation fills missing parameter
  slots with initialized `undefined` and rejects extras under the guarded source
  profile. Call frames, builtin signatures and native unwinding remain open.
- Ordinary scalar blocks and `if`/`else` lower with wide checked branch targets.
  Initialization joins include only paths reaching each read; both source branches
  are checked. Snapshot storage/work is bounded. Tests cover 70,000 local snapshots
  and 12,000 nested branches; full native scope/constant-folding parity remains open.
- Scalar `while` now reevaluates conditions through verified backward jumps and
  bounded execution fuel. Exit initialization conservatively permits zero
  iterations. Tests cover 70,000 local snapshots, long backward jumps and 12,000
  nested loops. Native watchdog timing remains open.
- Scoped scalar while `break`/`continue` preserve nearest-loop targets and
  terminating-path initialization joins. Break links share admitted instruction
  records; tests resolve 20,000 fixups and lower 12,000 nested contexts.
- Staged scalar `for` supports optional clauses and sends `continue` through the
  update. Update initialization merges continuing paths under existing quotas.
  Clause/constant-folding restrictions remain explicit; general GSC remains open.
- Owned legacy vector helpers now preserve component-wise arithmetic, numeric
  broadcast, exact equality and addition's separate mixed-type rejection. They
  match 19,852 recorded original-helper cases; a full object heap remains open.
- Verified primitive-kernel vectors now support construction, numeric/vector
  operators and copied locals/arguments under actual payload byte limits and fuel.
  18,116 native construction/operator cases execute through owned programs;
  source construction now uses the explicitly guarded primitive profile below.
- Primitive source vector triples evaluate components in explicit z/y/x order,
  with singleton grouping, original spans and existing quotas. Tests retain 70,000
  vector locals and 12,000-deep expressions. Pure scalar-return syntax stays narrower.
- Primitive vector indexing reads integer coordinates 0–2 with exact float bits,
  index-before-object evaluation and redacted type/range failures. Individual
  component writes stay unsupported under legacy rules; general indexable heaps remain open.
- Both source profiles preserve equality between two Undefined values and reject
  mixed/ordered Undefined pairs, checked against 402 actual C-helper outcomes.
- A separate verified primitive module API now executes direct/recursive calls
  using owned frames and global instruction/setup/memory budgets. Tests cover
  12,001 frames and 66,000 functions. Imports/methods/threads remain open.
  Excess arguments fail at executed calls; skipped call sites remain valid.
- Bound primitive source modules now compile local forward/recursive calls and
  preserve developer inclusion/visibility, source maps and whole-module budgets.
  Source tests cover 70,000 arguments/parameters, 66,000 functions and 12,000 nested
  call expressions. Imports, heap values, methods, builtins and scheduling remain open.
- Bound source also stores/passes/returns local `::function` references and invokes
  `[[expression]](...)` targets. Opaque references reject foreign/stale owners and
  share existing global runtime limits. All 2,664 actual C identity/type comparison
  cases pass; native compiler/VM, receiver/self, heap and full-mod gates remain open.
- Bound source calls `isint`, `isfloat`, and `isvector` directly without native
  plugins. Active script definitions shadow these pure builtins; wrong argument
  counts fail at invocation after argument evaluation. General builtins/game
  services, the complete value heap and full-mod compatibility remain open.
- Integer `switch` preserves default placement, adjacent labels, fallthrough,
  nearest break/continue ownership and initialization joins. Tests cover 66,000
  cases with balanced dispatch and 12,000 nested switches. Full int32 keys are an
  intentional expansion beyond native encoded-ID bounds; string switch remains open.

Function compilation returns `CompiledFunction` with `FunctionCompilationBudget`
and `FunctionCompileError`. `compile_scalar_return_function` selects the bounded
`PureScalarReturn` contract; `compile_primitive_function` selects
`PrimitiveStatements` without fabricated-argument execution. These restricted
profiles do not establish full GSC/module/mod support.
`compile_bound_primitive_module` separately consumes a validated local symbol index
and returns owned `CompiledModule` code/maps, with explicit declaration-to-entry
mapping. It processes local definitions without resolving imports or activating game state.

The protocol crate has no third-party dependencies, socket I/O, global state,
platform FFI, or dependency on the original server binary. Logging and process
signal dependencies are isolated above deterministic protocol/server services.

## Local query-only runtime

```sh
cargo run -p cod4x-server -- --query-only --bind 127.0.0.1:28960 --log-dir logs
```

Development builds use readable, automatically colored stderr by default;
release builds use JSON. Override with `--human-logs` or `--json-logs`, and use
`--color auto|always|never` for terminal style. Files always remain JSON.
`--run-for-ms 1000`
provides a bounded local run. The default bind is loopback, the mode must be
explicit, and discovery advertises a development server with zero player slots.
See [logging and operations](docs/logging.md) for event and retention contracts.

Verify an operator-supplied IWD without opening a network listener:

```sh
cargo run -p cod4x-server -- --verify-iwd path/to/content.iwd
```

This checks every original payload, including hidden duplicates, without extracting
files or logging their names/data. It reports counts and the legacy checksum and
returns a failure status for corrupt content. The command uses the library's
default configurable budgets; operator configuration for those budgets is future
work. Loading decoded assets/maps and checking pure-server admission remain open.

Inspect an unsigned version-5 fastfile envelope offline:

```sh
cargo run -p cod4x-server -- --inspect-fastfile path/to/map.ff
```

This validates compression integrity and reports declared payload/block/external
bytes without extracting files. Asset records, encoded pointers, geometry, and
scripts remain unvalidated; this command does not load a playable map. Signed
fastfiles are explicitly unsupported by this source baseline.

`--inspect-zone-prefix PATH` additionally checks the script-string table, inline
strings, and asset directory and reports their counts. References and asset bodies
remain unvalidated; root tables are not a loaded zone or a decoded gameplay map.

`--inspect-rawfiles PATH` decodes a raw-file-only collection and reports aggregate
usage. It rejects other asset types before returning content, opens no listener,
and executes no scripts. Names and buffer bytes remain outside diagnostics.

Inspect a loose legacy GSC file offline:

```sh
cargo run -p cod4x-server -- --inspect-gsc path/to/script.gsc
```

This checks structural syntax and reports counts or the original line/byte column
on failure. It also checks local function definitions and developer regions.
Use --gsc-developer enabled to include developer-only definitions in name lookup;
the default is disabled. All definition metadata remains retained. It does not
resolve imports/call expressions, validate body literal values, compile, or execute
scripts. Source bytes/paths stay outside logs. Default source/scanner/parser/
declaration/binding budgets apply; command-line budget overrides remain future work.

Exercise the legacy challenge/cookie exchange over UDP:

```sh
cargo run -p cod4x-server -- --handshake-probe --bind 127.0.0.1:28960
```

This mode verifies a return path and always refuses connects with a clear
gameplay-unavailable error. It does not admit players or authenticate accounts.
Its process key comes from OS entropy; logs report only aggregate categories.

## Source baseline

Reference: [callofduty4x/CoD4x_Server](https://github.com/callofduty4x/CoD4x_Server).
Revision: `ddd65fce1a1093017bad2dbb6840d75d11c0d56f`.
Exact provenance is recorded in [upstream.lock.json](upstream.lock.json) and
[NOTICE](NOTICE). The reference checkout is kept outside this repository.
Game content is not included.

## Development

Install the pinned toolchain with `rustup show`, then run from the repository:

Use only the normal `target` directory. On Windows run
`& .\tools\stop-dev-server.ps1` before rebuilding to stop a previous development
server after verifying its executable belongs to this repository.

```sh
python tools/check_repository.py
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo test --workspace --release --locked
```

Build API documentation with `RUSTDOCFLAGS=-Dwarnings` set in your environment:

```sh
cargo doc --workspace --no-deps --locked
```

Script tests use explicit `frontend` and `legacy` suites while leaf files stay
separate. Filter a module with `cargo test -p cod4x-script --test frontend -- expression_lists::`.
Repository policy ensures each test leaf is registered exactly once.

To reproduce the C fixtures, use Python 3.11+, Git, and GCC on a little-endian
host. The generator verifies the upstream revision and canonical source hash
before compiling selected functions into a disposable local oracle:

```sh
git clone https://github.com/callofduty4x/CoD4x_Server.git ../cod4x-reference
git -C ../cod4x-reference checkout --detach ddd65fce1a1093017bad2dbb6840d75d11c0d56f
python tools/generate_message_vectors.py ../cod4x-reference --check
python tools/generate_wire_vectors.py ../cod4x-reference --check
python tools/generate_huffman_vectors.py ../cod4x-reference --check
python tools/generate_obfuscation_vectors.py ../cod4x-reference --check
python tools/generate_content_vectors.py ../cod4x-reference --check
python tools/generate_user_command_vectors.py ../cod4x-reference --check
python tools/generate_bsp_vectors.py ../cod4x-reference --check
python tools/generate_zone_pointer_vectors.py ../cod4x-reference --check
python tools/generate_script_vectors.py ../cod4x-reference --check
python tools/generate_identifier_vectors.py ../cod4x-reference --check
python tools/generate_numeric_vectors.py ../cod4x-reference --check
python tools/generate_string_operation_vectors.py ../cod4x-reference --check
python tools/generate_arithmetic_vectors.py ../cod4x-reference --check
python tools/generate_lexical_tables.py ../cod4x-reference --check
python tools/generate_literal_vectors.py ../cod4x-reference --check
python tools/generate_syntax_tables.py ../cod4x-reference --check
python tools/generate_cookie_vectors.py --check
python tools/generate_archive_fixtures.py --check
python tools/generate_fastfile_fixtures.py --check
python tools/generate_zone_fixtures.py --check
python tools/generate_rawfile_fixtures.py --check
```

Omit `--check` only when deliberately regenerating and reviewing fixtures.
The C oracle is test tooling and is never linked into the Rust product.

## Design and completion criteria

- [Engineering standards](docs/engineering.md)
- [Architecture and future client](docs/architecture.md)
- [Compatibility contract](docs/compatibility.md)
- [Capacity limits and content expansion](docs/limits.md)
- [Source audit](docs/source-audit.md)
- [Milestones and acceptance gates](docs/roadmap.md)
- [Expanded GSC runtime requirements](docs/gsc-runtime.md)
- [Local validation record](docs/validation.md)
