# Assessment 0101: Native function names in discarded developer code

Date: 2026-11-01

Status: four additional selected native cases pass locally on Windows. Source
developer blocks remain gated; complete call/builtin emission does not run here.

The 0099 family now additionally executes pinned EmitFunction with compile mode
two, local/file-qualified function ASTs, and owned/borrowed name flags. Its full
external service boundary is explicit. Name lookup, canonical filenames, variable
allocation, linking, opcode positions and value services fail closed if reached.
Only bounded modeled name-release services are available on the selected path.

All four cases bypass name lookup/linking and preserve existing modeled output.
Owned local names request one release; owned file-qualified names request two.
Borrowed names request neither. File-qualified discard decrements the pending
file counter independently of that ownership flag. Local discard does not.
No code target, executable placeholder, runtime function reference or file load
is produced. These observations concern EmitFunction's mode-two branch, not
general EmitCall's earlier builtin probing or all compiler error ordering.

The twelve original developer-list outcomes remain byte-identical. Two added
admission tests reject altered output/diagnostics, missing name releases, releases
of borrowed names and a collapsed pending-file counter. Mutation helpers now bound
each edit to its exact C/F scenario. The generator checks sixteen cases and ten
admissions in this family; seven scope families contain 56 admission tests total.
All are temporary freestanding native tools, independent of Cargo target storage.
Driver O0/O2 outputs agree, the regenerated fixture checks, and repository/diff
policies pass. Hosted CI/Linux remain unrun locally; Rust is unchanged from the
complete Windows ADR 0098 gates (407 frontend/273 legacy).

Disabled developer source must not simply resolve every name as executable code:
that would reject references this selected native discard path never links. A
future validation-only compiler path must have no publishable unresolved target,
preserve argument/body checks and resource accounting, discard its temporary
instructions/literal/label/field rows, and avoid mutating outside transfer/fact
chains. General builtin/call probing, source activation, scheduling, game services
and complete GSC remain open.
