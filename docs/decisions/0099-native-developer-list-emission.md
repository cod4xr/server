# Assessment 0099: Selected native developer-list emission

Date: 2026-11-01

Status: twelve selected native cases and eight trace-admission regressions pass
locally on Windows. Source developer statements remain rejected.

The bounded freestanding x86 driver executes actual pinned
EmitDeveloperStatementList and Scr_FindLocalVarIndex instructions. Existing
five scope families and their drivers/fixtures remain unchanged. This new family
has independent expected scope tables, compilation-mode visits, opcode requests,
retained modeled output, string-service counts and compiler-global restoration.
Driver O0/O2 outputs must agree before fixtures are admitted.

Ten cases pair enabled/disabled developer-script mode with an empty list, new
private name, existing public name, a later planned parent name that has no current
cell, and a modeled aborting body. Two additional cases enter with nested compile
mode one or two. Planned parent/child tables and body statements are supplied
explicitly. These selected native instructions do not run local-name analysis,
the parser, general body dispatch, return emission, VM, value freeing or error
unwinding. All output/string/temp/diagnostic services are bounded models.

Observed rules agree with the independent expectations:

- Disabled bodies are still visited in compile mode two. Their modeled output
  rewinds to the saved position; earlier output survives. Treating disabled source
  as unchecked text would miss this validation path.
- Enabled bodies run in mode one. A new private slot requests create/remove;
  an existing public slot requests neither. A name planned later in the parent
  does not preserve a cell privately created at this earlier point.
- Disabled output rewind bypasses the enabled final private-removal request.
  Its temporary child planning/emission state can retain a larger created count.
  This is discarded compiler state, not proof of a live runtime owner leak.
- Neither path publishes private initialization/names or a modeled body abort
  into the parent. Normal paths restore compile mode and source-position globals.
- Nonzero entry compile modes report the recursion diagnostic before body visits
  or table/global changes. The diagnostic service returns at this selected native
  epilogue; native exception/longjmp behavior is not executed.

Fixture rows distinguish opcode requests from retained output. Native output
alignment, opcode optimization and actual string reference ownership do not run.
The shared harness admits exact external services, uses temporary native tools,
and never creates an additional Cargo target directory or Rust FFI boundary.

Eight admission tests reject incomplete/duplicate/unknown inventories, missing or
wrong disabled visits, lost sentinel/output rewind, unrestored globals, omitted
private removal, falsely public planned names, cleanup on existing/aborting cases
and nested-mode body execution. CI includes this generator and admission suite;
hosted CI/Linux remain unrun locally.

Source activation requires a separate review of actual native developer local
analysis, mixed control/return/break/continue paths, inherited definition visibility,
validation of disabled bodies without host authority, iterative metered child
contexts, definite initialization versus physical cells, private failures, complete
resource limits and package/CLI execution. These selected twelve cases do not
authorize a shortcut around those requirements or close a full-GSC workstream.
