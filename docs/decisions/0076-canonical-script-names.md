# ADR 0076: Canonical virtual script names

Date: 2026-11-01

Status: implemented naming component; cross-file linking/loading remains open.

## Decision

Add a NUL-free ASCII LegacyScriptPath view and independently owned
CanonicalScriptPath under binding/script_paths. Match the selected native
CreateCanonicalFilename helper's naming rules: ASCII lowercasing, collapsing
separators, skipping leading separators and control bytes, and preserving dots,
spaces and one trailing slash. Controls restart the helper's segment state:
"a\n/b" becomes "ab", whereas "a/\n/b" becomes "a/b". Dot segments are not resolved
and extensions are not removed. Empty canonical identifiers remain representable
by this low-level helper; a later module catalog must apply its own admission.

This is a virtual identity, never filesystem permission, a host path validator,
file loading, an interner ID or complete GSC linking. Module/catalog lookup must
remain separate from content-root/path-traversal policy. Reject embedded NUL rather
than silently selecting a prefix. Reject non-ASCII outside the safe profile;
native ctype behavior for negative signed chars/locale is not inferred.

## Ownership, work and limits

View construction validates complete borrowed input and counts canonical bytes
without allocating. Its iterator borrows source bytes and emits canonical bytes
without temporary strings. Owned copying preflights complete input/output/work,
uses try_reserve_exact and publishes only a complete Vec payload. Source may then
be dropped. Hash/equality use canonical bytes; a linker must prepay their linear
raw-input visits separately. Native helper output/error buffers, wrapper interners
and source spelling do not escape into runtime IDs or error diagnostics.

Input bytes, canonical bytes and work have named configurable budgets (defaults:
1 MiB input/output, 8,388,608 visits). Validation/counting charges two complete
input visits; copying charges raw scan plus canonical output bytes. Failed admission
preserves previously charged work and publishes no partial name. Retained usage
counts owned payload, excluding fixed fields, allocator/spare capacity and RSS.
Debug/errors redact paths and expose only categories/counts. No unsafe code or new
crate/dependency is added.

The native wrapper uses a 1,024-byte stack buffer; a separate function-handle path
also asserts MAX_QPATH. Those are source/profile boundaries, not authoritative Rust
storage limits. Tests retain 70,000 complete name bytes. This does not establish
that native loaders or clients accept those names.

## Evidence and alternatives

generate_script_path_vectors.py executes the actual helper at O0/O2 in the C locale
and checks 579 ASCII outcomes against an independent segment/regular-expression
model. Error callbacks intercept with setjmp, not native unwinding. Native partial
error-buffer prefixes remain helper evidence; Rust rejects before publishing data.
574 NUL-free cases replay through Rust; five embedded-NUL cases intentionally reject
as catalog identities. script_path_reference.py rejects 20 selected-role mutations
and distinguishes helper rules from the wrapper buffer/interner. CI/syntax checks
include these guards; 99 structural syntax replays remain unchanged.

Seven Rust tests also cover 4,096 randomized split/join model names, canonical hash
identity/idempotence, source-drop ownership, diagnostics privacy, all non-ASCII
bytes, 70,000-byte names and every failed work prefix/exact input/output budget.

Reusing filesystem path normalization would incorrectly resolve/reject or rewrite
native virtual-name quirks. Using native fixed buffers would inherit unwanted
ceilings and partial writes. Borrowed canonical iteration plus admitted ownership
separates identity, storage and later host I/O. It prepares explicit package
namespaces without claiming qualified calls/includes/import resolution are implemented.

Windows workspace debug/release, all-target Clippy, rustdoc, formatting and policy
passed: 275 frontend, 229 legacy tests, 13 script unit tests and two doctests;
861 source files/eight crates, largest source 287 lines. Native filename 579
outcomes, 20 source-role mutations and 99 syntax replays passed. Linux/hosted CI
remains unrun locally. Cross-file compilation is the next step; full GSC/S5 remains open.
