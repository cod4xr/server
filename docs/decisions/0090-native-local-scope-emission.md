# Assessment 0090: Executed local-slot and branch-transfer emission

Date: 2026-11-01

Status: selected Windows evidence accepted; Rust physical-scope implementation
and source indexed writes remain open under ADR 0088.

Assessment 0089 executes local-name planning but cannot infer physical alias
lifetimes from its declaration tables. This step executes the actual pinned
Scr_FindLocalVarIndex, EmitIfStatement, EmitIfElseStatement, EmitBreakStatement
and EmitContinueStatement instructions alongside the planning helpers. Thirty-six
selected cases reproduce exactly at both C driver optimization levels and satisfy
an independent table/formula model before fixture publication.

## Boundary

The shared freestanding x86 runner still verifies the reference revision/hashes,
extracts complete native functions/data and checks the exact external service set.
An explicit emission profile shares the temporary process/toolchain path; no Rust
FFI, dependency or Cargo target is introduced. AST setup moves into one shared C
leaf without changing any of the 34 planning fixture bytes.

Native slot creation, name reordering, initialized-bit transfer, branch joins,
created/public count updates and removal-count selection execute. Condition/body
dispatch, canonical IDs/string-reference services, opcode output, code storage and
temporary allocation remain modeled. The body model accepts plain assignment or
a two-assignment block and invokes the actual native slot helper. Nonconstant
condition emission returns a modeled dynamic result. Unsupported paths fail.

Output records opcode requests, selected slot-name/removal operands, private block
state and source-position call counts. Native EmitOpcode optimization and native
code alignment are not executed; the output is not a compatible bytecode image.
No parser, full expression/statement emitter, VM, allocator reuse, error unwinding
or actual object/string/array freeing runs. Continue/break tests use synthetic
created/public block-count profiles, not complete source loops. Windows passes;
Linux/hosted CI remains unrun.

## Results

- A fresh one-sided branch local emits CreateLocalVariable and removal of one
  local on branch exit. Two fresh locals remove two. An inherited parameter does
  not create or remove a slot.
- A later direct declaration of the same spelling does not preserve the earlier
  private branch cell: the removal still occurs. An already materialized outer
  cell survives. This resolves the analysis-only ambiguity recorded in 0089.
- Common if/else declarations survive in the parent and carry initialized facts
  across the join. Different branch declaration orders normalize names consistently.
  Disjoint branch locals receive separate cleanup. A prior outer local survives
  while the branch's additional local is removed.
- Continue subtracts the current block's public count. Break subtracts the
  designated break-target block's public count. Both reset the current block's
  created count using its own public count before recording their transfer.
  These are separate inputs, demonstrated by 24 synthetic count combinations.

Seven one-sided cases, five if/else cases and 24 transfers have complete expected
state and request traces. Six admission regressions reject missing/duplicate/
unknown cases, incorrect later/prior/common-join cleanup, swapped transfer count
rules and damaged initialized/name/created state. Existing planning, removal source
mutation, structural and suite-registration guards remain required.

## Next implementation

Use a metered iterative source prepass to plan ordered block declarations,
fallthrough/return/break/continue outcomes and shared public names. Track physical
local lifetime separately from definite-initialization facts and stable wide
published slots. Generate explicit alias release on each relevant control exit;
normal blocks alone do not define a new lifetime boundary. A later declaration
cannot rescue an earlier private alias, and an existing outer owner cannot be
cleared merely because a branch assigns its name again.

Do not activate source writes until branch, loop, switch/developer and transfer
cleanup is implemented or explicitly rejected with a reviewed source profile.
Test alias counts/COW/key order through subsequent writes, every failure prefix,
roots under collection, exact work/storage and deep/wide control flow. Existing
typed mutation and frame-owner cleanup are components of this work, not proof of
source lifetime parity. No complete GSC or join workstream closes here.
