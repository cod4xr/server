# ADR 0014: Bounded source-grounded legacy lexing

Date: 2026-10-31

Status: accepted and implemented; full-workspace validation recorded separately.

## Context

The pinned tree contains lexer/parser tables and actions in _scr_yacc.asm rather
than an editable source grammar. New language keywords can break old identifiers:
foreach, for example, is an ordinary identifier in this baseline. The expanded
language needs a separate opt-in/versioned mode above an evidenced legacy path.

## Decision

Recover character equivalence/meta maps, DFA tables, terminal codes, and mode
changes into reviewed generated Rust data. Validate reachable byte transitions,
fallback termination, action shapes, and dimensions during regeneration. Export
only the 256 character-map entries, excluding following unlabeled addresses in the
assembly listing. State/table/token-kind widths are this profile's grammar data,
never engine identifier, object, memory, or string-pool limits.

LegacyLexer borrows admitted SourceText and performs allocation-free iteration.
Keep every raw span/trivia piece, preserve spelling/delimiters, and distinguish
code/comment modes. Literal decoding, numeric conversion, identifier interning,
syntax parsing, module resolution, and execution are later stages. Terminal codes
are not runtime values. Do not activate modern keywords or silently lowercase input.

Bound returned lexemes, transition/fallback attempts, and individual lexeme bytes.
Count lookahead/repeated work. A configured limit fails explicitly; errors contain
fixed kinds and byte offsets without private text. Stop permanently after the first
error. EOF is explicit rather than modeled as unsafe input-buffer overread. Reject
embedded NULs instead of silently dropping a source tail; report unclosed block
comments at their opening offset. These are explicit safer malformed-input rules.

Debug exposes classifications/counts only. No filesystem, logging installation,
native code, mutable global lexer state, runtime allocation, or hidden prefix bytes
enter this layer. Future parsers must finish validation before publishing modules;
successful lexing is not valid syntax or executable bytecode.

## Alternatives and migration

A newly guessed grammar could accept/reject old inputs incorrectly. Linking the
original parser would keep native state/ABI constraints. Discarding trivia or
normalizing source would lose tooling/source-span information. Data recovery plus
an explicit bounded scanner gives a concrete frontend stage. An original executable
or equivalent full lexical/parser conformance run remains an open evidence gate.

## Evidence and limits

Eight tests include 724 independent Python source-table/action replays, exact
terminal codes, preserved identifiers/case, trivia partitioning, borrowed spans,
empty/EOF/NUL/comment outcomes, redacted Debug, resource boundaries, work accounting,
100,002-byte raw literals, and all 65,536 two-byte inputs. Boundary tests caught an
initial comment-mode mapping error; mode changes are now recovered from action
instructions rather than manually duplicated. Regenerated replays and independent
unclosed-comment/partition tests pass after correction.

The replays are source-derived, not execution of original x86 code, and are not
counted as original-C function fixtures. Long raw literals do not prove string
value decoding, interning, or VM capacity. A full lexer/parser trace matrix, literal
semantics, syntax/AST, compiler, bytecode validation, scheduler, modern features,
and stock-mod/gameplay compatibility remain open.
