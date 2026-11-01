# Assessment 0079: Closed-package include linking

Date: 2026-11-01

Status: selected archival source roles guarded; closed include execution later follows ADR 0080.

## Scope and evidence

ADRs 0077/0078 implement explicit closed source catalogs, qualified calls and the
offline command. Includes remain rejected by the command, while the bound-symbol
API deliberately carries no directive metadata. Before admitting includes, preserve
the pinned compiler's separate direct-definition and imported-definition roles.

`script_include_reference.py` guards ordered blocks in the hashed archival compiler
and type identities in scr_vm.h. Sixty-five in-memory removal/reordering/type
mutations are rejected. The checks observe these selected roles:

- Include names pass through native canonical filename creation and queued
  dependency records carry an explicit include flag.
- Queued dependencies use Scr_LoadScriptInternal; a missing script has a distinct
  compile error. Duplicate flagged canonical includes have a dedicated rejection.
- Import enumeration accepts function objects with code-position entries and
  skips entries already marked VAR_INCLUDE_CODEPOS. Imported definitions are
  marked separately in the destination namespace and their pending references
  pass through LinkThread.
- SpecifyThreadPosition rejects an already populated destination definition,
  distinguishing source-located and positionless collisions.
- EmitFunction rejects an already imported code position on its immediate
  resolution path. This requires keeping imported targets distinct from direct
  definitions when resolving later qualified names.

These are selected source roles, not execution of native compilation, import
graphs, loader state, error cleanup, ABI or VM. Existing package/function/call-link
guards separately cover local-only builtin fallback and developer references.
The standalone guard, integrated syntax/99 structural replays and repository
policy pass (896 sources/eight crates/largest 287 lines). No Rust behavior changes
in this assessment; the latest complete Windows Rust gates are commit a7ea4ed.

## Implementation candidate

Add an explicit opt-in directive slice to a supplied module definition; its empty
default keeps existing caller contracts. Resolve includes only within the supplied
canonical catalog. Inventory every active direct definition first, then materialize
one-hop imported aliases. Imported aliases must not become new exports. This
permits cyclic source dependency graphs without recursively expanding imports or
requiring the Rust host stack to walk a transitive closure.

Keep direct/qualified/unqualified lookup distinctions and developer visibility.
Reject repeated canonical include targets and destination function collisions,
including unused definitions, before publication. Do not silently overwrite a
local definition. Investigate builtin-versus-include call precedence and local
function-reference behavior against the emission/deferred-linking paths before
activation; a generic search-order change can alter existing scripts.

Admit aggregate include edges, complete alias rows, namespace/name comparisons,
staging/sort/hash work and overlapping scratch before allocation. Use wide module/
function/alias indices and configurable quotas, with private numeric directive
locations and atomic failed package publication. Reuse the single verified image,
global execution budgets and owned source maps. Include aliases must not change a
function body's original file-local namespace or its source module ownership.

## Required acceptance

Cover one-hop imports, non-transitive exports, ordinary/developer selection,
direct/builtin/import collisions and precedence, qualified references to imported
names, duplicates, unknown files, forward/cyclic graphs, all declaration/source
maps, source-drop ownership, wide aliases/modules/functions and every insufficient
work/storage scope. Run existing package/source/process regressions and Windows
gates. Update CLI directive handling only after the import policy is implemented
and its behavior has evidence. Animation directives remain separately unsupported.

Automatic dependency discovery, mount/GSX/GSC precedence, source preprocessing,
native load-cache/deferred ordering and stock/mod execution are later acceptance
gates. The candidate does not choose them or claim full GSC compatibility.
