# ADR 0032: Admitted scalar local storage and explicit argument binding

Date: 2026-10-31

Status: accepted; implementation and validation tracked separately.

## Context

Typed statements expose locals but the scalar executor has no local storage or
arguments. The native compiler registers at most 64 locals, tracks initialization
bits, and rejects uninitialized reads in Scr_FindLocalVarIndex. A slot initialized
to Undefined is different from one never initialized. The existing kernel's proof
covers control-flow/stack shape, not source binding or definite assignment.

## Decision

Extend typed internal instructions with checked wide local load/store indices.
Add an immutable function layout with parameter-prefix and total-local counts.
Require parameters <= total locals and checked host-address conversion. Verify
all local operands, including unreachable instructions, against this layout before
copying code. Admit total locals independently of instruction/stack metadata;
retain the layout inside the owned proof. The verify convenience supplies the
zero-parameter/zero-local layout; execute supplies an empty argument slice while
retaining the proof's local layout.

Invocation binds an exact argument slice to the parameter prefix under this
internal contract. This is not a claim about native GSC missing/extra-argument
semantics; a legacy call adapter requires separate evidence. Copy supplied scalar
arguments into private local storage. Other slots begin uninitialized. Stores
initialize slots, including a stored Undefined; uninitialized loads return a
located error rather than fabricate a readable Undefined. Local access still
checks bounds at runtime, and each instruction consumes fuel as before.

Admit argument count, local count/checked logical bytes, and operand-stack budgets
before allocating local storage. Use fallible complete reservation and bounded
initialization; count admission bounds initialization work separately from
instruction fuel. Local count/storage scopes exclude caller arguments, allocator
rounding, and RSS. Return counts/logical bytes on success and admitted local count
on failure, without names/values. Each invocation owns isolated fresh slots and
never mutates the caller's arguments or another invocation.

## Consequences and alternatives

Initializing all slots to readable Undefined would erase an important distinction.
A native-sized local array or packed short slot would reintroduce obsolete
capacities. Duplicating the interpreter for local functions would split numeric,
fuel, branch, and stack-proof behavior. The shared typed kernel instead retains
explicit layouts/budgets and checked local access. Its proof does not establish
source definite assignment; that still belongs to local binding/control-flow
lowering. The existing restricted source compiler does not acquire locals merely
because the kernel can store them.

No native/serialized ABI, function-call stack, name binding, object heap, scheduler,
general source compilation, legacy arity policy, or host services are introduced.
Future call frames must use separately budgeted storage rather than the native
32-frame/2,048-value/local arrays.

## Evidence

Keep existing scalar/source tests. Exercise argument ownership and exact arity,
layout/local bounds including unreachable code, store/load/Undefined versus
uninitialized reads, control-flow-selected initialization, independent invocations,
numeric failure/fuel accounting, count/byte admission, zero fuel before allocation,
and local slots/arguments beyond old limits. Bounded hostile streams must reject
or remain within their proof/fuel; they must not produce invalid private state.
