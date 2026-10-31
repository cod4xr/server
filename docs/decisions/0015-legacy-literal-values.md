# ADR 0015: Bounded legacy literal values without old scratch buffers

Date: 2026-10-31

Status: accepted and implemented; full-workspace validation recorded separately.

## Context

Raw lexing preserves source bytes but does not produce values. The pinned parser
uses a fixed string scratch buffer, decodes r/t/n escapes (other escapes copy the
escaped byte), then interns through native string tables. Numeric actions call
C-locale-style sscanf conversion routines. Copying those storage/locale/undefined
conversion assumptions would limit an expanded runtime and complicate validation.

## Decision

Decode one admitted literal lexeme into LegacyLiteral: undefined, legacy number,
or owned ordinary/localized bytes. True/false retain legacy integer values. Unary
signs and token boundaries remain parser responsibilities. This does not validate
an entire source file or introduce modern numeric/escape syntax.

String decoding preserves exact source action behavior, including r/t/n and the
default escaped-byte rule. Unknown escapes do not introduce Unicode/hex/octal
decoding. Count decoded bytes, admit encoded/decoded budgets, then allocate fallibly
and return a complete value. No fixed 8191-byte stack buffer, implicit terminator,
string-table registration, or packed string handle enters the result. Empty strings
need no payload allocation; Debug/errors omit source values.

Parse decimal integers with checked signed-int32 arithmetic and finite decimal
binary32 values without ambient locale. The lexer determines accepted spellings:
exponent notation, for example, is not a single float token in this baseline.
Reject integer overflow and non-finite float results explicitly rather than claim
compatibility with undefined/CRT-dependent overflow. Modern wider numeric types,
spellings, and escapes require the versioned language path in the GSC contract.

## Alternatives and migration

Interning while scanning would couple syntax to native tables and publish partial
state. Reusing a fixed scratch buffer would retain a capacity cut. Implicit UTF-8
repair would change legacy byte strings. Values now own their decoded payload and
can later move into a quota-accounted constant pool. Full runtime handles, AST,
binding, bytecode, host APIs, and execution remain separate work; no existing VM
state is migrated by this phase.

## Evidence and limits

Four tests include 545 source-action/CRT replays: 522 ordinary/localized string
cases and 23 numeric results. Escape dispatch/replacement and numeric format
strings are checked against pinned assembly. Numeric calls run the host CRT in
explicit C locale/nearest rounding with int32/binary32 shape assertions. They are
not execution of the original parser or interner and are not counted as original
function fixtures. Exact binary32 ties/subnormals/underflow/max finite values and
decimal leading-zero integers match this defined replay profile.

Further tests preserve unknown escapes, binary bytes, empty/localized values,
source immutability, redacted Debug, booleans/undefined, wrong token kinds, rejected
overflow, and encoded/decoded exact budgets. A 100,000-byte decoded string passes
without truncation. This proves value decoding, not string interner/VM capacity,
whole-program syntax, compiler semantics, or stock-mod/gameplay compatibility.
