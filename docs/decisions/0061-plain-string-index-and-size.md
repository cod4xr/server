# ADR 0061: Plain-string byte indexing and size

Date: 2026-11-01

Status: accepted for explicit heap-backed execution and bound source.

## Problem and decision

Legacy string reads are not Unicode character operations. Scr_EvalArray requires
an integer index and returns one byte before strlen's terminating NUL. Negative,
out-of-range and noninteger indices fail; localized strings are not indexable.
Scr_EvalSizeValue likewise uses strlen for plain strings. Its native array/object
branches do not make those values implemented in Rust, and vectors/localized
strings do not support size. Preserve these rules for the available value subset.

Extend the existing typed Index instruction to plain strings through an explicit
KernelHeapContext, retaining bottom-to-top index/object order. Add unary Size
with complete stack-proof/fuel admission and fixed private errors. Strings remain
immutable: indexing allocates a one-byte value without changing the source. Take
the byte before pressure collection and retain all current/caller/cache/external
roots until allocation succeeds. Collapse operands only after success. Frame
publication now consumes the instruction's checked stack effect for either unary
or binary heap operations, rather than assuming every heap operation is binary.

Charge index metadata and the possible prefix scan before examining bytes; charge
the independently admitted result allocation and any root/collection work through
the existing global context. Size conservatively prepays the whole retained
payload scan, including bytes after an early NUL, and allocates no heap value.
Owned bytes after NUL remain preserved for storage/output, while these legacy
operations expose only the prefix. A length outside int32 fails with SizeOverflow
instead of truncating; a modern wide-size/buffer API remains separately versioned.

Bound source emits Size after visiting its object and maps failures to the .size
token. Existing reference reads already evaluate index before object and retain
the bracket span. Isolated scalar source profiles remain explicit. Unsupported
field reads still prevent complete module publication; tests formerly using size
as an unsupported form now use an actual unsupported field. Invalid localized
size is a located runtime type error, not a compiler failure.

## Alternatives and consequences

Unicode code-point/grapheme indexing would change ordinary scripts. Treating the
complete binary allocation as a C string would change embedded-NUL behavior.
Borrowed slices would require a new value/ownership type; owned one-byte results
reuse the current checked heap and resource boundaries. No interner, length cache,
new crate/dependency, native ABI or general object authority is introduced.
Repeated size calls scan again; cached lengths need separate measured justification
and retained-metadata admission. This change does not implement string writes,
string switch/coercion, arrays/objects, scheduler or full legacy mods.

## Evidence

896 O0/O2 outcomes execute the original string index/size functions with explicit
terminated-byte/ref/error services: 28 indexed bytes, eight sizes and 860 errors.
Every row replays through Rust checked arguments. Cases include empty strings,
non-UTF-8/UTF-8 bytes, embedded NUL, int32 boundaries, all available primitive
categories and a 70,000-byte value. Reference/output services are synthetic; error
callbacks return to the helper. No native allocation/interner/ref cleanup, error
unwinder or original compiler/VM execution is claimed. Selected access/emission/
dispatch guards reject 45 scoped mutations; native object size branches are
guarded source evidence only, not Rust object support.

Three additional kernel tests cover exact/one-less work, atomic failed result
allocation, unary proof admission and collection with an object held only on the
operand stack. Three source tests cover index-first errors/maps/privacy, wide
sizes/indices and raw UTF-8 bytes, plus 998 collections in a 1,000-call loop under
four slots. The 4,096 hostile module inventories now include index/size operations.
Two real process tests cover escaped byte/size output and private located failures;
the existing localized-size process test now checks runtime rejection. The runnable
string_access_demo.gsc returns string b"Rt" using the normal target directory.
Full workspace Windows policy/formatting, warnings-denied Clippy/rustdoc and debug/
release tests passed (239 frontend/140 legacy tests, 659 sources/eight crates).
The 99 structural replays and independent predicate/vector-index fixtures remain
unchanged. Linux/hosted CI/full-mod acceptance stays open.
