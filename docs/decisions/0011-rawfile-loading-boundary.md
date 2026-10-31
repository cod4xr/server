# ADR 0011: First typed IW3 asset reader: raw-file-only collections

Date: 2026-10-31

Status: accepted and implemented; full-workspace validation recorded separately.

## Context and source findings

The pinned rawfile.h has a twelve-byte 32-bit record: name word, signed length,
buffer word. Load_RawFile in src/asmsource/_db_load.asm reads that record in the
temporary stream, switches to block four, loads an inline or direct-offset name,
and consumes length + 1 serialized buffer bytes whenever the buffer word is nonzero.
That buffer word is a presence flag, not a standard offset/sentinel decoder input.
Scr_ReadFile_FastFile uses strlen, making a validated terminator necessary before
exposing a raw buffer as a C-compatible script source.

Load_RawFilePtr scopes block zero, aligns its record, and rewinds temporary space
on return. Encoded -2 additionally allocates a four-byte alias slot in block four
before reading the body. Both the root pointer cell and inserted slot eventually
contain the registered asset pointer. Other encoded root headers read aliases;
they are not direct references to a serialized RawFile body. Registration/pool
semantics are another layer and must not be conflated with parsing records.

## Intended implementation

The concrete raw-file-only collection reader precedes a generalized graph loader.
Reject unsupported root asset kinds upfront, preserve directory order, null entries,
and aliases, and return borrowed immutable names/buffers only after the entire
supported collection validates. Never skip unknown bodies or activate a zone.
Use explicit logical temporary/virtual allocation accounting; do not allocate
declared blocks or cast serialized words into native pointers.

Keep alias bindings private to the decode owner and bind only validated prior
targets. Null root cells are known from the complete directory and may be aliased
before their turn. Root header cells and inserted alias slots have separate logical
addresses. Reject other unbound or mistyped aliases in this supported subset;
full recursive asset graphs require their own staging/relocation decisions.
Retain known inline string regions in allocation order and resolve direct names
only through their validated borrowed spans, including suffixes and empty strings.
Prefix inline strings can seed those regions; raw unresolved words cannot become
native addresses. This subset must state unsupported reference cases explicitly.

Admit name/data/work/metadata budgets with wide checked arithmetic and fallible
growth. Name/data contents and host paths stay out of Debug/errors/logs. No global
registry or engine IDs are assigned, no duplicate-name override semantics are
claimed, and no script executes. Asset registry publication follows separately
after typed graph/import validation. Existing encoded catalogs remain independent.

## Validation and operator inspection

Twenty independent synthetic envelopes cover inline/null entries, backward root
aliases, -2 inserted aliases, shared/suffix name references, empty/binary buffers,
temporary-block reuse, virtual alignment, budgets, malformed lengths/terminators,
truncation, unsupported kinds, forward/unbound aliases, and excess body bytes.
No real stock map can be claimed loaded by this subset because its material,
model, sound, collision, and other asset kinds remain unsupported. Original-C/ASM
full loader differential evidence and registration semantics remain open gates.

Six content tests cover exact names/binary buffers, zero-copy/shared/suffix views,
prefix-name references, null/empty distinctions, temporary-block reuse, alignment,
all sample body truncations, deterministic mutations, and exact/one-less budgets.
Three process tests exercise `--inspect-rawfiles`, failure exits, mode separation,
aggregate accounting, and omission of names/buffers/paths from diagnostics.
The command opens no listener and writes no extracted content. Staging aliases/name
regions are released after complete decode; logical peak metadata accounting
excludes prefix metadata, spare capacity, hash/allocator overhead, and source bytes.
