# ADR 0068: Compound and postfix ordinary struct fields

Date: 2026-11-01

Status: implemented for the ten existing compound operators and integer-only
postfix increment/decrement on ordinary bound-source fields.

## Decision and evaluation order

Reuse checked field reads/writes, immutable name tables and existing value
operators. Compound assignment evaluates the receiver and reads its old value,
then evaluates RHS and the operator, then evaluates the writable receiver again.
The second receiver may refer to a different object. Do not collapse these two
evaluations into a single selected lvalue. An operator failure must skip the second
receiver. This follows selected pinned EmitStatement_790/500 and field-expression
roles; existing local compound order remains unchanged.

Postfix evaluates its writable receiver once. Retain a copied receiver on the
operand stack while reading/updating the field, then exchange the top pair for
StoreField. The small checked Swap instruction consumes/produces two values,
does not allocate and preserves stack height. Proof checks reachable underflow;
runtime checks the count before permutation. This is an internal typed instruction,
not a claim about a native bytecode opcode or serialized ABI.

Postfix retains the existing strict integer-only, wrapping word behavior. Compound
arithmetic retains existing numeric/vector/string domains and errors. Neither
profile creates local bindings for field names. Compiler tasks, code, names,
identifier visits, source maps, shared proof and invocation stack/fuel remain
explicitly admitted. The original update/operator token locates runtime failures.
The retained receiver and pending old value participate in normal complete-root
collection through calls and allocations. No speculative lvalue arena or crate
is required for these ordinary struct operations.

## Native evidence and intentional boundary

script_field_update_reference.py rejects 41 scoped mutations to selected compound
read/RHS/operator/write roles, receiver evaluation counts and postfix dispatch/
ordering. It also checks existing field and integer-update profiles. Syntax-table
generation and CI run the new guard. This is selected source evidence, not native
compiler/VM execution, error unwinding or reference-cleanup parity. Existing value
operator fixtures remain the separate numeric/vector/string evidence.

On malformed postfix input, Rust reads and rejects the unsupported value before
creating any absent field. The native writable-reference path can create an
Undefined child before reporting an increment/decrement type error. The Rust
profile deliberately preserves missing/stored Undefined and existing fields on
these invalid operations; native temporary allocation/error cleanup is not claimed.
The whole invocation remains nontransactional: completed receiver/RHS effects,
collections or earlier instructions can persist after failure.

## Verification and consequences

Source tests execute all ten compound operators, integer word boundaries, vector
updates and functions selecting different read/write receivers. Postfix tests
distinguish single receiver evaluation from compound's two evaluations. Error tests
check skipped second receivers, unchanged values/missing fields, exact operator
spans and private diagnostics. String-pressure tests retain localized/cache roots
and a caller-stack-only old value whose field the RHS deletes before collection.

A 12,000-edge source chain executes compound/postfix reads and writes through more
than 65,535 instructions, then fails a separate compile partway through lowering
under a 6,000-task budget without recursive ownership. Nine compiler/module/proof
resources accept exact limits and reject one less. Typed Swap tests check order,
underflow, exact stack bytes/count and fuel. A real CLI process executes updates
with readable logs and numeric output while suppressing private keys/paths.

No dependency, crate, native runtime linkage or additional target directory is
introduced. Array COW and indexed lvalues, methods/self, entity fields, general
object operators, scheduling, imports and full stock/mod acceptance remain open.
This does not close S5 or establish a joinable server.

Windows workspace debug/release tests, warnings-denied Clippy/rustdoc, formatting,
repository policy and diff checks pass. Script suites contain 265 frontend/180
legacy tests, six private units and two doctests; all three field process tests
pass. The 41 update mutations and 99 structural replays regenerate. Policy checks
751 source files/eight crates with a largest source of 287 lines. Linux/hosted CI
and native game interoperability remain unverified.
