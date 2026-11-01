# Assessment 0089: Native local-name planning and removal

Date: 2026-11-01

Status: selected native evidence accepted on Windows; Rust scope lowering and
source indexed-write activation remain open under ADR 0088.

The current source compiler keeps function-wide binding slots. With writable
arrays, retaining a branch-only alias can change later copy-on-write selection
and sibling order. Frame-exit cleanup alone cannot establish compatibility.

## Executed evidence

`generate_local_scope_vectors.py` extracts the complete pinned x86-32
Scr_CalcLocalVarsVariableExpressionRef/Scr_CalcLocalVarsStatement and their two
expression-evaluation dependencies. NASM assembles the actual instructions and
referenced native jump tables/strings. A freestanding GCC driver executes 34
synthetic AST cases; C driver O0/O2 outputs must agree byte-for-byte. The assembly
itself is identical between those runs; they are not two native optimization
builds. The committed fixture records every parent/allocated block and auxiliary
allocation. Case identity, shape, inherited names/initialization and selected
branch/loop consequences are checked independently before fixture publication.

Cases include one-sided/disjoint/common if branches, returning branches,
ordinary blocks, nested branches, different declaration orders, while/for loops,
break/continue, nested loops and later direct outer assignments. Names and input
initialized flags are synthetic. Nonconstant conditions avoid unavailable value
services; encountering an unsupported service exits unsuccessfully.

The driver models bounded AST storage, temporary allocation, globals and process
output only. It is not the native parser, emitter, bytecode interpreter, allocator,
error unwinder, string interner, reference counter or GSC engine. In particular,
`created`/initialization fields remain seeded analysis inputs: these snapshots do
not demonstrate emitted cleanup or actual alias release. Later direct assignment
is another analysis call, not an emission pass or a complete whole-function plan.

The runner supports Windows/Linux x86 hosts without a CRT or 32-bit libc headers.
It requires GCC with freestanding `-m32` code generation and NASM; Windows also
requires GNU dlltool/ld with i386 support. Windows uses only Kernel32 process I/O;
Linux uses x86 process write/exit syscalls. All generated assembly, objects, import
libraries and executables live in one checked temporary directory, removed on
completion. No Cargo target/cache is created. Output, allocations, ASTs and process
duration are bounded. Windows execution passes; Linux/hosted CI remains unrun.

## Guarded source roles

`script_local_scope_reference.py` checks eight original 0x17 emission sites across
if/if-else, break/continue, switch and developer blocks. It preserves the selected
created/public subtraction, byte count emission and created-count reset roles.
The opcode header and actual VM dispatch lead to a byte count, local-stack/count
reduction and the RemoveNextVariable loop. Ninety-eight scoped role/order/header/
dispatch mutations must reject. These are source guards, not executed VM cleanup.
Structural grammar checks and upstream CI include the guard; CI installs NASM
and regenerates the trace fixture. Six admission regressions reject missing,
duplicate, malformed or semantically changed traces without invoking native code.

## Consequences for Rust

An ordinary block is not independently sufficient to end local lifetimes. Common
if/else names can join the parent; returning paths do not contribute ordinary
fallthrough facts. Loop fallthrough, continue and break have distinct planning
effects, including differing for body/update blocks. A later outer declaration
also requires whole-function planning rather than clearing every newly encountered
name at the first closing brace. Do not implement cleanup by source braces alone.

Next, establish emission-time transfer and removal for these paths, then add a
metered iterative Rust plan for physical local lifetimes and every control exit.
Keep initialization facts separate from owner lifetimes. Use wide logical slots
and configured budgets; native 64-name/byte-count limits are evidence about the
legacy profile, not new global Rust limits. Prove ownership cleanup under failure,
branch/loop merges, deep nesting and storage pressure before source writes activate.

No Rust runtime behavior changes here. Complete Windows Rust acceptance remains
the ADR 0088 checkpoint: 345 frontend/257 legacy tests, fifteen private script tests
and two doctests. Complete GSC, nested array ownership, scheduling and join gates
remain open.
