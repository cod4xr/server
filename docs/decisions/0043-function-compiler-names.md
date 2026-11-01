# ADR 0043: Function compiler names before broader primitive source values

Date: 2026-10-31

Status: accepted; implementation and validation tracked separately.

## Context

The original source compiler admitted scalar returns, then local statements and
control flow. Parameterized programs now accept vector arguments, and vector
source construction is next. Public/implementation names containing Scalar no
longer describe the growing function compiler. This standalone project has no
released Rust SDK; ordinary client/content compatibility is not a Rust symbol ABI.

## Decision

Rename owned source results/accounting/errors to function compiler terminology:
CompiledFunction, FunctionCompilationBudget/Usage/Resource, FunctionCompileError/
Kind and FunctionSourceProfile. Give the two implemented source contracts explicit
names PureScalarReturn and PrimitiveStatements. Name their entry points
compile_scalar_return_function and compile_primitive_function. Rename compiler
leaf modules/private data from scalar to function, retaining descriptive names
for the scalar-return and primitive-function entry points. Update shared test
helpers/imports and current documentation; preserve historical ADR evidence.

Do not add compatibility aliases/deprecated wrappers for an unreleased internal
API, extra profiles, crates or a second implementation. Preserve accepted syntax,
all runtime behavior, quotas/accounting, diagnostic categories/locations/privacy,
proofs and source
maps. PrimitiveStatements still does not imply full GSC, modules/activation,
objects/calls/scheduler/host authority. This change defines no protocol/content/
serialized bytecode change or new language feature.
Diagnostic prose/type labels follow the new function terminology.

## Evidence required

Run the existing frontend/legacy suites and complete Windows workspace gates;
new tests duplicating the mechanical rename are unnecessary. Check stale symbol/
module references and explicit helper/test registration. Keep files below policy
limits, reuse the normal target, and record native/full-GSC/Linux gates still open.
Source vector constructors remain separately specified and validated work.
