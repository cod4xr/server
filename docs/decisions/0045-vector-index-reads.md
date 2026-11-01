# ADR 0045: Legacy vector index reads before general indexable heap values

Date: 2026-10-31

Status: accepted; implementation and validation tracked separately.

## Context

Primitive source can construct/operate on vectors but cannot read coordinates.
The pinned Scr_EvalArray accepts only integer vector indices 0..2 and returns a
float copy; Scr_EvalArrayRef explicitly rejects individual vector component writes.
The parser's index node stores object then index; the selected compiler evaluates
index first, object second, and the VM passes the top object plus prior index to
Scr_EvalArray. This differs from simply assuming left-to-right source traversal.

## Decision

Add a fixed-size vector read helper returning LegacyNumber::Float, with typed
integer-index/range failures that never retain the attempted index or coordinates.
Preserve exact component bits, including NaNs and signed zero. Do not coerce float
indices or add mutable vector components. Original inputs remain owned copies.

Extend the primitive kernel with Index: its explicit internal stack contract
consumes bottom-to-top index/object and produces one result. Initially only vector
objects are indexable; unsupported objects fail with a fixed type category, and
invalid indices retain fixed vector-domain errors. Share existing proof, actual
payload-byte/local admission, copied arguments and global instruction fuel. This
is not a native/serialized bytecode ABI or general arrays/strings/heap support.

Extend PrimitiveStatements reference reads through a focused leaf for plain names
and index syntax. Schedule index, then object, then Index, retaining source bracket
and descendant operation spans. Keep initialization/identifier/task/code/resource
admission and profile ownership shared. PureScalarReturn stays narrower. Indexed
assignment/compound/postfix targets remain unsupported; future array lvalues and
modern mutation semantics require their own contracts.

## Evidence required

Execute the actual pinned Scr_EvalArray vector branch with bounded stubbed object/
pool/ref/string/error services; never claim those services/native error indices/
cleanup were reproduced. Guard selected parser node slots, compiler operand
traversal/emission and VM/helper operand roles, plus legacy write rejection.
Test all valid indices, signed/out-of-range integers, float index classes, exact
component bits, copy/isolation/redaction, source loops/reconstruction/mapping and
index-before-object error order. Cover verifier stack effects, quota/fuel edges,
wide/deep source/typed input, failed cleanup and source profile regression.
General indexables/heap, calls/scheduler/host services, native/full-GSC/mod/Linux
acceptance remain open.
