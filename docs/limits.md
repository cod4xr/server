# Capacity limits and content expansion

The rewrite must remove inherited internal ceilings for assets/models, images,
scripts, memory, maps, entities, and configstrings while retaining compatibility
paths for existing clients/content. The user reports that previous ApX development
required removing more than 50 characters and reducing 50 prestige levels to five
because of image-related limits. That content loss is a primary design concern.
The exact exhausted pool in that earlier project has not been diagnosed here;
client/server logs and manifests would be required to establish its cause.

## Separate storage from representation

The authoritative engine should retain the entire content catalog using bounded,
growable storage and wide internal identifiers. Capacity is an explicit resource
budget chosen for hardware/workload, with accounting and actionable exhaustion
errors. It is never an unchecked allocation or a legacy `MAX_*` array copied into
the new architecture. Server-only state can exceed a client's visible namespace.

Each peer's codec/content view must validate that its IDs, counts, strings, assets,
and state can be represented. Keep legacy indices in a per-peer compatibility
mapping; never cast/truncate an engine identifier to make it fit a packet field.
Visibility/interest filtering can reduce replicated objects without removing
server objects, but it is not a universal solution for maps or client-required
assets. Residency, concurrent registration, catalog size, and wire namespaces
must be accounted for separately.

A custom client can negotiate larger catalogs, asset namespaces, script resources,
state counts, and suitable formats. Existing peers use an explicit compatible
content view or fallback where semantically valid. Content requiring incompatible
geometry/protocol features must be identified before admission/download. Backward
compatibility means preserving supported old behavior and content; it cannot give
an unchanged client an encoding or renderer it does not implement.

No automatic truncation, silent content removal, fabricated success, or downgrade
that changes game rules is acceptable. Mixed legacy/extended sessions need an
explicit policy and tests for equivalent gameplay where claimed.

## Initial audited values, not new engine ceilings

Source baseline is upstream.lock.json. These values describe the pinned server;
they do not establish a universal limit for every client or CoD4X version.

| Area | Pinned source evidence | Expansion boundary |
| --- | --- | --- |
| Entities | `entity.h`: 10 index bits, 1024 values, reserved world/none IDs | Wide engine IDs; legacy snapshot projection; extended peer codec |
| Players | `sys_cod4defs.h`: 64; `msg.c`: six-bit client indices | Admission, simulation, snapshots, client arrays, voice and scoreboard |
| Configstrings | `sys_cod4defs.h`: 4883 entries, typed category ranges | Dynamic engine registry plus profile-specific wire tables |
| Registered models | `MAX_MODELS = 512` in configstring layout | Distinguish registry namespace from loaded model pool |
| Registered materials | `MAX_MATERIALS = 256` in configstring layout | HUD/icon registration and client-render resource budgets |
| Registered sounds | `MAX_SOUNDS = 256` | Loaded audio assets and network-visible indices are separate |
| Asset pools | `db_load.cpp`: model 1000, material 2048, image 2400 defaults | Growable per-kind stores; client pools remain independent |
| Existing pool overrides | `DB_ParseRequestedXAssetNum`: selected pools can grow up to 65535 | Investigate current override coverage; do not assume every pool is fixed |
| Script identifiers | `cscr_variable.cpp`/`scr_vm.h`: packed masks and 16-bit fields | New runtime representation, language/behavior compatibility, host APIs |
| Compiler temporaries/branches | `_scr_compiler.asm`: 32-value constant stack and 16-bit expression jump patches | Budgeted compiler work/storage and wide internal control-flow locations |
| Function locals | `Scr_CalcLocalVarsVariableExpressionRef`: count compared with 0x3f before insertion | Budgeted local slots and checked wide indices; binding semantics need separate evidence |
| Script call/value stacks | `scr_vm.h`: 32 function frames, 2048 value/local-variable slots | Separately budgeted frames/value storage; scheduler and full VM remain open |
| Script strings | `cscr_stringlist.cpp`: 32768-entry hash allocation | Growable owned slot storage tested at 70,000 live strings; VM integration/interner semantics remain open |
| String concatenation | `Scr_EvalPlus`: 8192-byte scratch, maximum 8190-byte payload | Budgeted owned helper/store concatenation implemented; automatic VM accounting/rooting remains open |
| Script file handles | `scr_vm.h`: ten handles | Managed, quota-accounted resources and sandboxed filesystem APIs |
| Map/collision formats | `cm_local.h`: several 16-bit counts/indices | Validate old formats; extended geometry needs compatible new readers |
| Paths | `q_shared.h`: 64-byte virtual and 256-byte OS paths | Internal path handling, archive validation, legacy name encoding |
| Messages | `net_game_conf.h`: 128 KiB; normal packet 1400, fragments 1300 | Legacy codec bounds and MTU policy, not global memory limits |

Image count, material count, HUD-visible material configstrings, fastfile contents,
and renderer residency are different limits. Increasing a server image pool alone
does not prove that legacy clients can load/display the full character and
prestige catalog. Inventory the entire dependency chain and actual peer versions.

## Required implementation and evidence

- Add per-kind/current/peak resource metrics and explicit allocation budgets.
- Define stable internal asset/entity/script IDs independently of serialized IDs.
- Validate complete content manifests and dependency closures before activation.
- Diagnose the exact limiting namespace in an exhaustion report; never emit only
  a generic "image limit" when the failed resource is known.
- Build a regression catalog containing at least the original target of 50
  prestige tiers and all desired character assets, without reducing the source
  catalog. Test legacy-view decisions separately from extended-client exposure.
- Test beyond the old boundaries for each expanded subsystem, including mixed
  peer profiles, reconnects, unload/reload, missing capabilities, and exhaustion.
- Measure real memory, load time, frame time, download cost, and GPU residency on
  supported clients. "Unlimited" is not a resource-management strategy.

Current protocol constants remain compatibility budgets. Growable encoded asset
ownership, bounded IWD reading, and IBSP container metadata are implemented; their
tests do not establish decoded asset/geometry expansion. The script VM, fastfile
graphs, semantic map loading, full snapshot projection, extended capabilities,
and custom client required for these goals remain implementation work.
