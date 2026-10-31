# ADR 0013: Immutable bounded script source and byte locations

Date: 2026-10-31

Status: accepted and implemented; full-workspace validation recorded separately.

## Context

An expanded compiler/debugger needs exact source spans, useful diagnostics, and
large-source support without old global buffers or printing script contents through
ordinary logs. Text storage, lexical acceptance, module identity, and human display
columns are separate responsibilities.

## Decision

Add SourceText and borrowed SourceLine views in cod4x-script/source. Borrow immutable
raw bytes and own only a fallibly allocated line-start index. Admit source byte,
line-count, and logical metadata budgets before allocation. Preserve tabs, NULs,
non-UTF-8 data, and exact endings; a later lexer decides what syntax permits. No
host path, global module registry, or implicit NUL terminator is introduced.

Expose one-based line and byte-column coordinates with wide external offsets.
LF starts a new line; CRLF stays intact, while standalone CR remains content.
Newline bytes belong to the preceding line. EOF is valid, including an empty final
line after trailing LF. Empty source has one empty line. Views offer full bytes and
content excluding the recognized ending without repair.

Positions are relative to the source used for lookup. A future owning module/source
registry supplies scoped identities; these coordinates are not global handles.
Byte columns are not Unicode scalar/grapheme widths or tab-expanded display cells.
Dedicated tooling performs display conversions in its own context. Debug/errors
expose counts/categories only. No parser, compiled source map, debugger, execution,
or original-diagnostic parity is claimed by this container.

## Alternatives and migration

Normalizing strings/endings would invalidate byte spans and lose binary input.
Copying each line would duplicate source data. A global source buffer constrains
concurrent modules. A borrowed index supports frontend input with explicit
ownership. Module storage and compiled source maps remain separate work; no
existing VM/source ownership is migrated.

## Evidence

Six tests cover mixed endings, byte columns, binary/UTF-8/NUL input, tabs, empty and
trailing lines, EOF/spans, rejected ranges/offsets/lines, exact/one-less budgets,
zero-copy views, independent sources, redacted Debug, and 70,001-line indexing.
A separate simple scan checks every offset of 256 deterministic byte patterns.
These test the new source API, not the original formatted-line ABI.
