# Engineering standards

## Authority and interpretation

Follow [Google's Rust Style Guide](https://google.github.io/styleguide/rustguide.html),
then the public Rust Style Guide and Rust API Guidelines where they do not
conflict. This is the Rust guide, not a mechanical translation of Google's C++
rules. Reviewed against the published guide on 2026-10-31.

Chromium/Fuchsia-specific build systems, flat source-layout rules, executors,
mutexes, imports, and test frameworks do not apply to this Cargo project.
Use conventional Cargo workspaces and Rust's test harness. Stable rustfmt
enforces 100-column formatting and the pinned 2024 style edition. Nightly-only
formatter settings are not enabled; reviewers enforce module-level import
grouping and lowercase hexadecimal spelling where stable rustfmt cannot.

## Structure

- Aim for at most 250 lines per source file. The hard ceiling is 400 lines,
  including comments, tests, and embedded source. CI rejects larger handwritten
  `.rs`, `.py`, `.c`, `.cpp`, `.h`, `.js`, `.ts`, `.tsx`, `.ps1`, and `.sh` files.
- Split by responsibility and dependency direction, not arbitrary line chunks.
  Root modules declare modules and re-export APIs; implementations live in leaf
  modules named after their main type. Keep integration tests outside production
  modules. Split tests by behavior when needed.
- Create crates for meaningful ownership, dependency, platform, or trust
  boundaries. Create subfolders for cohesive groups within a crate. Do not create
  empty crates or an unrelated `utils` dumping ground.
- Script integration tests use explicit frontend/legacy suite roots containing
  module declarations only. Leaf files and fixtures retain separate ownership.
  Repository policy requires every plain #[test] file to be registered exactly
  once without gating, including leaves added in subfolders; see ADR 0026.
  Use ordinary same-name module declarations for root leaves and explicit paths
  for nested leaves; helpers stay in module folders. Run the registration-guard
  regression tests after harness/policy changes; see ADR 0084.
- Each crate inherits package metadata and workspace lints. New dependencies
  require an explicit allowed dependency entry, rationale, license review,
  maintenance review, and consideration of build scripts and transitive costs.
- Avoid speculative traits, generic frameworks, macros, shared mutable globals,
  and reference counting without an actual use case.

## APIs and implementation

- Explicit imports; one source module per import statement. Qualify external
  free functions with their module; import type names directly. No glob imports.
- Use descriptive types and lifetimes, newtypes for distinct domains, enums for
  ambiguous boolean parameters, and named fields for structured data.
- Prefer ownership and borrowing. Use borrowed byte slices for wire data, not
  implicit UTF-8 conversion. Document public APIs, resource bounds, errors,
  mutation guarantees, and byte/bit order.
- Use `Result` with specific errors for recoverable input failures. Error
  messages and logs must not disclose packet secrets, credentials, or private
  configuration. Callers must propagate parse errors before mutating live state.
- Production `unwrap`, `expect`, explicit panic, TODO, unimplemented, and debug
  macros are denied. Narrow test-only `expect` allowances require a reason.
  No broad lint suppression or unsafe escape hatch is permitted.
- Safe Rust is mandatory in current crates. A future FFI or SIMD requirement
  needs an ADR, a separately reviewed boundary crate, justified invariants,
  `SAFETY:` comments, and evidence that safe alternatives are insufficient.
- Bound untrusted lengths, counts, allocation, CPU work, queues, cache lifetime,
  and fan-out before expensive processing. Explicitly define malformed input,
  resource exhaustion, timeout, shutdown, and overload behavior.

## Verification and review

The required checks are repository policy, formatting, Clippy with warnings
denied, debug and release tests, and rustdoc with warnings denied. CI defines
Windows and Linux jobs; local Windows results do not establish Linux success.
Cargo.lock and the exact Rust toolchain are versioned. Workflow actions are
pinned by commit; workflow credentials have read-only permissions.

Protocol changes require independent upstream vectors and malformed-input
tests. Round trips alone do not prove compatibility. Add property and fuzz
coverage as parsers grow; run sanitizers/Miri for applicable boundaries and
long-running adversarial workloads before claiming a public-server milestone.

Performance claims require reproducible workloads, CPU/memory/latency profiles,
hardware details, and comparison to the pinned server. No unmeasured claim of
being faster or better is acceptable. Preserve observable compatibility without
copying memory-unsafe behavior or undocumented architectural debt.

For substantial changes, review responsibilities, contracts, failure modes,
compatibility evidence, operational behavior, test sufficiency, and documentation.
Architecture changes require an ADR stating the problem, decision, alternatives,
consequences, migration, and validation. No large-file exception may be added
just to avoid separating responsibilities.
