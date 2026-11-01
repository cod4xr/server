# ADR 0036: Explicit scalar legacy script-argument adaptation

Date: 2026-10-31

Status: accepted; implementation and validation tracked separately.

## Context

The internal scalar invocation API requires exact argument count. The pinned
script compiler creates parameter locals before safe-setting supplied arguments;
the VM's cached safe setters skip assignment at PRECODEPOS and checkclearparams
reports extra arguments. New variable creation preserves Undefined. This motivates
an explicit missing-Undefined/excess-error script profile; it does not describe
builtin/service signatures, expression evaluation/push order, or full frame/error
unwinding behavior.

## Decision

Add an explicit scalar legacy script-argument invocation entry point. Supplied
values are already in source parameter order. Reject count greater than parameters;
copy supplied values into the parameter prefix and initialize only the missing
parameter suffix to Undefined. Nonparameter locals remain uninitialized. Reuse
the same local count/checked byte and operand-stack/fuel admission before allocation.
Fill the private local vector directly, with no extra padded argument buffer or
unaccounted temporary value array. Initialization visits each admitted slot once.

Keep existing exact-argument and empty-argument entry points under their existing
contracts. Every invocation owns fresh slots and preserves caller arguments even
when type/numeric/fuel/initialization errors occur. Report fixed errors/counts/
source locations, never argument contents. Native marker values/pointer layouts
do not enter typed code, and no serialized/call ABI is defined.

Source guards check compiler parameter walks, create-local, safe-set skip/consume,
marker/checkclearparams and extra-argument error roles plus undefined-variable
creation evidence. They do not execute the original handlers. The profile applies
only to supported scalar script invocation; calls/frames, receivers/objects,
builtins, threads, native error cleanup and complete GSC/mod parity remain open.

## Evidence

Test every supplied-prefix length, missing Undefined versus uninitialized locals,
excess rejection, exact-API regression, copied float bits/caller isolation, source-
compiled parameter functions, fresh invocations, wide missing parameter prefixes,
local count/byte/stack/fuel boundaries and huge checked failures before allocation.
Keep current source/kernel/reference checks and fault-inject the new source guard.
