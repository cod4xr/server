# Assessment 0091: Native loop scope emission and condition correction

Date: 2026-11-01

Status: selected Windows evidence accepted; Rust physical-scope lowering remains
open. No source indexed writes activate.

Ten new cases execute actual pinned EmitWhileStatement/EmitForStatement alongside
the existing slot, branch and transfer helpers. They cover dynamic-condition
fallthrough/break/continue, common branches, mixed exits, nested loops and an
omitted for condition. Independent expected block state, opcode requests and
condition-dispatch counts must match both C driver optimization levels.

## Corrected earlier input

The original six for planning cases (15/16/17, later=0/1) in assessment 0089
supplied a raw expression where the native AST requires a present optional
condition wrapper: tag 0x41, then the expression. Other tags select the omitted
condition path. The new emission harness exposed the mistake because no condition
check was requested. These six earlier inputs and their fixture results are now
corrected, not preserved as compatibility evidence. The 28 other planning cases
and all 36 branch/transfer emission cases remain byte-identical.

The correction removes one auxiliary 4096-byte allocation from each affected
planning case. Case 17's body abort/public metadata changes for both later modes;
without a later declaration its private body name no longer joins the parent.
A shared C AST-construction leaf now owns while/for setup in both planning and
emission drivers, avoiding duplicate condition shapes. The planning admission
model explicitly checks all three for parent-name outcomes and has a regression
rejecting the former incorrect case-17 export.

Condition-emission counts are independently asserted: each dynamic for condition
is dispatched once while the omitted condition is never dispatched. Regression
tests remove/change both the condition request and count; they must fail.
This is an explicit exception to earlier fixture-preservation instructions for
the six identified erroneous synthetic inputs, not permission to rewrite other
recorded outcomes when an implementation disagrees.

## Executed scope effects

Dynamic while fallthrough/continue can materialize public local slots before the
condition, preserving them around the loop. A local assigned only before break
can remain private and is removed on that transfer. A mixed nested branch uses
the correct break/continue ownership and initialized-bit positions after name
reordering. Nested while cleanup belongs to the inner loop.

For emission distinguishes initializer, body and update blocks. Continuing paths
carry body initialized facts into the update; a private body local on the tested
dynamic-condition break path is removed. With an omitted condition, the tested
assignment-before-break name instead joins the parent and no private removal is
requested. A plan that treats these loops identically loses a native distinction.

The modeled statement dispatcher now routes bounded synthetic branches/loops/
transfers into the actual selected native helpers. It still models ordinary
assignment by invoking the native local-slot helper. Its recursive dispatch is
bounded to eight synthetic levels; it is not the production Rust compiler and
does not authorize source-host recursion there. Statement lists, code output,
ASTs, allocation and execution time remain explicitly bounded.

## Validation boundary

Windows passes all ten independently modeled loop cases at O0/O2, regenerated
corrected 34-case planning and unchanged 36-case emission fixtures, nineteen
admission regressions, 98 source mutations, 99 structural replays and repository
policy. The same two Rust test targets/eight crates/normal target are preserved;
no Rust behavior changes. Complete Rust workspace gates remain ADR 0088.

The condition/body/opcode/string/allocator service boundaries from 0089/0090
remain modeled. Opcode-request streams are not compatible bytecode; actual native
EmitOpcode optimization/alignment, parser, VM and value freeing do not execute.
Constant-condition analysis, switch/developer emission and native error unwinding
remain outside these cases. Linux/hosted CI remains unrun.

Next, account for those remaining source-scope profiles and implement an iterative,
budgeted Rust declaration/lifetime plan with explicit cleanup on every supported
edge. Keep definite initialization separate from ownership and preserve wide
slots. Full GSC, mixed/nested ownership, scheduling and joins remain open.
