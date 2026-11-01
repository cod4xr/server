# ADR 0078: Offline supplied-file GSC execution

Date: 2026-11-01

Status: implemented; required Windows verification recorded below. Full GSC/S5 remains open.

## Problem and scope

ADR 0077 supplies a closed multi-file compiler and runnable fixed-source example.
The existing server command accepts one source file. Developers need to execute
an explicitly supplied package through the real process, existing log formats,
typed stdout and shared runtime budgets without writing a Rust caller.

## Decision

Add an offline `--execute-gsc-package ENTRY_NAME` development mode with repeatable
`--gsc-module NAME PATH` inputs. Names identify virtual modules; paths are explicit
operator-supplied local files, never derived from script paths. Keep the existing
single-file inspection/execution modes. Package module options require package
execution, developer mode remains explicit, and network options stay illegal.
Reject missing/duplicate/invalid names, missing active main, directives and every
unsupported active body before execution or stdout publication.

Read admitted regular-file snapshots. Admit aggregate source bytes, source index,
tree/declaration/symbol work and metadata across all files as separate frontend
scopes, then use ADR 0077's global compiler/proof admission. Defaults are resource
policies; do not import legacy filename/function/include chain ceilings. Use
fallible allocation and bounded module-row storage. Retain indexed sources through
execution for numeric module/local-declaration/byte/line/column diagnostics.

Reuse existing heap setup, execution, result formatting and metrics for both
single-file and package images. Do not create a second interpreter, logging path,
dependency, crate or target directory. Developer stderr keeps its readable format;
explicit/release/file JSON keeps machine-readable events. Logs and errors carry
numeric positions/categories/counts, never private names, paths or source/return
payloads. Explicit stdout remains the existing one typed result.

The package runner has its own cohesive folder for CLI admission, aggregate
frontend budgets, source phases and orchestration. A small borrowed ScriptImage
enum shares execution/result/metric handling with the single-module path. Parsing
and compiler staging drops before runtime; source snapshots and line indices stay
live for locations. No source-derived host paths are constructed. The CLI action
boxes its fixed-size run settings once rather than expanding empty help/version
variants with the larger package configuration.

Snapshot admission shares the source default's 64 MiB across all files. Every
index/parser/scanner/declaration/symbol resource that accumulates across files is
clamped to remaining phase admission and debited after success. Parser peak
metadata is conservatively accumulated, bounding all retained trees plus each
tree's temporary peak; this may reject some inputs earlier than a retained-only
policy. Live parser stack limits, per-token/name limits, body/literal scopes and
whole-image proof remain separately scoped. Module-count admission bounds frontend
Vec record rows; all variable row allocation is fallible. Logical quotas exclude
allocator rounding/spare capacity and RSS. CLI quota overrides remain open.

Canonical CLI names are owned under ADR 0076 before source I/O. The raw catalog
byte quota counts complete original spellings, including discarded controls;
canonical compaction cannot bypass it. Main selection compares owned canonical
bytes, visiting at most the admitted catalog-name payload, then uses active local
declaration mapping instead of falling back to function zero. Every supplied
active body is compiled even when unreachable from main.

Compilation events add aggregate source/index/parser/scanner/declaration/binding
metrics alongside namespace/compiler/proof counts. Runtime events use the common
global frame/fuel/heap counters and the package profile. Failure events identify
numeric module/source coordinates when the failing phase supplies them; directive
and missing-main errors identify a module without inventing a byte coordinate.

## Alternatives and consequences

An automatic directory/dependency loader would need source precedence, include,
preprocessing and filesystem traversal contracts that this component has not
established. Explicit repeated inputs provide reviewable local authority without
claiming that loader. A manifest parser would add another format/dependency and
change-source policy; CLI inputs reuse the existing parser and logging lifecycle.
Keeping two runtime functions would risk differing roots, output and accounting;
the private image enum supplies only the required program/location/profile access.

## Validation

Eleven real-process tests exercise direct/indirect cross-file calls and recursion,
module/function case rules, source maps, developer visibility, heap values,
duplicate/unknown identities, unsupported directives/bodies, empty/missing entry,
resource failure, file errors and conflicting CLI settings. Human/JSON stderr,
flushed JSON files, all relevant typed result categories and private path/name/
payload diagnostics are checked. Two unit tests admit exact/one-less resources in
17 aggregate frontend scopes and exhaust module/raw-name CLI budgets independently
of Windows command-line length. Two individually valid 33 MiB snapshots reject the
combined 64 MiB admission before parsing. Existing single-file/query tests remain
part of workspace verification.

Run the repository example from its root in PowerShell:

```powershell
cargo run --locked -- --execute-gsc-package demo/root `
  --gsc-module demo/root examples/gsc/package/root.gsc `
  --gsc-module demo/library examples/gsc/package/library.gsc
```

Observed Windows debug result: `integer 1001`, two modules/two functions, 552
compiler steps, 4601 peak compiler metadata bytes and 22,022 runtime steps. The
command uses normal heap defaults; ADR 0077's Rust example separately forces
small-heap collection. These component observations are not a performance claim.

Windows formatting, all-target Clippy, debug/release workspace tests (including
eleven package process/two private frontend/catalog tests, 288 script frontend/
229 script legacy/13 private script tests and two script doctests), rustdoc with
warnings denied and repository policy pass. Policy covers 895 source files/eight
crates; the largest source remains 287 lines. Filesystem mount precedence,
automatic dependency loading/includes, source arrays/scheduling, game services,
stock/mod and Linux acceptance remain open.
