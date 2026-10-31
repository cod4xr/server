# cod4x-rust

A standalone Rust rewrite of the CoD4X dedicated server, designed to share
protocol and simulation code with a future custom client. Existing CoD4/CoD4X
clients, maps, and mods are compatibility targets. ApX-specific behavior will
live above the reusable engine and require explicit extension negotiation.
Inherited engine limits are removal targets: internal storage uses configurable
resource budgets, while old client/wire bounds stay inside compatibility profiles.
The engine must preserve full content catalogs rather than require asset cuts.

**Status: protocol foundation and runnable query-only development runtime.**
There is no gameplay, authentication, decoded map loading, scripting VM, renderer, or
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
  entity/client index codecs. Another 104 fixtures come from original functions.
- Socket-free channel sequencing and fragmentation in `cod4x-transport`, with
  cancellation, bounded reassembly, duplicate/reordering tests, and reference
  packetization. Wider engine IDs remain separate from legacy peer namespaces.
- Static Huffman compression/decompression verified against 257 codebook entries
  and 32 original C samples, preserving padding behavior without out-of-bounds reads.
- A query-only UDP application with deterministic discovery services, bounded
  per-IP/global reply budgets, shutdown signals, and real process/socket tests.
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
python tools/generate_archive_fixtures.py --check
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
- [Local validation record](docs/validation.md)
