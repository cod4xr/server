# Assessment 0100: Selected native developer local-name analysis

Date: 2026-11-01

Status: twenty selected native planning cases and eight trace-admission regressions
pass locally on Windows. Source developer statements remain rejected.

A separate freestanding family executes actual pinned CalcLocalVars instructions
for synthetic developer-list AST tag 0x2d. The selected helper retains its native
dispatch tables, branch/loop/developer analysis and bounded modeled allocation.
AST/name/global services remain modeled; the parser, emitter, VM, value freeing
and exception/error unwinding do not execute. The original five scope families
and Assessment 0099's twelve emission cases remain unchanged.

Ten scenarios each run with and without a later direct declaration of b: one
private name, two names, common if/else, disjoint if/else, return, nested developer
list, while break/continue and for break/continue. Independent expected tables
distinguish abort, created, public, total and initialized fields. Fixture admission
requires O0/O2 agreement and every complete row, including auxiliary allocations.

Ordinary developer bodies keep their new names private to their child. Common
branch names join into that child, not the function parent; disjoint branches keep
their own child tables. A later parent declaration does not retroactively enlarge
the developer child's original public prefix. A return marks the developer child
as aborting without marking the outer parent. Nested developer lists are planned
here; their emission-time recursion diagnostic is separately established by 0099.

Loop transfer analysis crosses the developer child. In the dynamic-loop cases,
break does not export the private developer name, while continue contributes that
name to the enclosing loop's continuation union. For-loop initializer, body and
update names retain their different order and scopes. These are local-name
planning facts; they do not prove runtime initialization, execution of disabled
code, actual cleanup or a reason to publish a private name on ordinary fallthrough.
The compiler mode is normal during these selected planning inputs; enabled and
discard-mode emission behavior remains separately covered by 0099.

Eight admission regressions reject incomplete/duplicate/unknown inventories,
unrecognized allocations, parent name leakage, retroactive public-prefix changes,
collapsed common/disjoint branches, invented parent aborts, merged break/continue
joins and incorrect for-loop name order. CI includes the new generator and suite;
hosted CI/Linux remain unrun locally. Seven native scope families now contain
54 trace-admission tests. Existing source mutation/structural/suite policies remain
in force; latest complete Rust acceptance remains ADR 0098 (407 frontend/273 legacy).

Source activation still needs checked developer-mode propagation into each module,
validation-only lowering for disabled lists, inherited developer call visibility,
iterative metered child planning/emission, separate initialized/physical cells,
returns and loop transfers, complete label/literal/table/code quotas, private
failures and package/CLI acceptance. More complex nested control and emission
profiles may require additional native evidence. No full-GSC workstream closes.
