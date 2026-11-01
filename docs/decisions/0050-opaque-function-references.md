# ADR 0050: Opaque function references and indirect primitive calls

Date: 2026-10-31

Status: accepted; implementation and validation tracked separately.

## Context

Bound primitive modules resolve named source calls but cannot store/pass/return
legacy function references or invoke double-bracket targets. The pinned header
uses OP_GetFunction 0x15, OP_ScriptFunctionCallPointer 0x51 and VAR_FUNCTION 9.
The compiler evaluates arguments before the indirect target expression, and the
VM checks/pops the top function value before entering the existing call path.
Native function equality compares code-position identities. Raw pointers and
module-local numeric indices alone cannot safely identify copied references across
independent module instances or after a module is dropped.

## Decision

Mint a checked wide identity for each completed immutable kernel module. A small
atomic counter holds identity allocation only, never game/asset/script state; it
must fail instead of wrapping/reusing identities. Allocate after complete proof
and retain failed verification work if identity space is exhausted. Use an opaque
copyable function reference containing that module identity and a wide index.
Construction stays inside checked module access and verified reference instructions.
Debug omits both components. This is data, not a native pointer, serialized ABI,
host authority or ownership that keeps a module alive.

Add internal FunctionReference and CallIndirect instructions. Verify every static
reference target and all reachable stack effects against the complete inventory;
standalone proof rejects unresolved reference/indirect-call instructions. Admit
argument-count-plus-target arithmetic before it can overflow, even in unreachable
code. Indirect invocation consumes the top reference after argument evaluation,
checks its type/module identity, and reuses the existing private call frames,
runtime arity checks, global fuel/setup/live-payload budgets and error locations.
The initial profile invokes references only within their owning immutable module;
foreign/stale references fail explicitly rather than resolving in another module.

Add the copied reference value to primitive data without choosing a heap/collector.
Copies/locals/arguments/returns remain private; actual size_of-based byte admission
must reflect any value/instruction layout growth. Explicit language equality uses
identity equality/inequality, rejects other function comparisons and preserves
existing numeric/vector/Undefined rules. Rust data equality remains distinct.
Execute actual pinned comparison helpers with synthetic 32-bit identity labels,
mapping labels bijectively to valid Rust function indices. That tests equality/
type rules, not native code allocation, function pointers or invocation/cleanup.

Bound source now lowers local function-reference expressions and indirect immediate
calls, preserving argument-before-target evaluation and rightmost argument order.
Reuse canonical local resolution and ordinary/developer reference policy. Keep
external paths, receivers, thread calls, builtins and isolated function profiles
explicitly unsupported. Do not add a callback/host/scheduler framework or resolve
foreign modules implicitly. Source maps and all existing compiler budgets remain
required; no unaccounted argument/reference array or recursive IR is introduced.

## Validation and limits

Test immutable identity uniqueness/nonreuse/exhaustion, same/cross-module reference
equality, copied/default/returned references, invalid types/owners, source factories/
higher-order calls, argument/target error order, visibility and redaction. Cover
all operand/proof/fuel/setup/count/byte limits, malformed/unreachable huge counts,
wide indices/argument prefixes, deep source/recursive execution, hostile streams
and fresh recovery. Guard selected locked parser/compiler/VM/helper roles with
mutation tests. Preserve the existing source/kernel/oracle/workspace gates.
These are owned typed execution and selected native-helper/source-role evidence;
native parser/compiler/VM execution, receiver/self semantics, cross-module loading,
closures/captures, heap values, scheduler/host services, full mods and Linux remain
open. No dependency, native/serialized ABI or alternate target directory is added.
