# ADR 0083: Explicit script root context

Date: 2026-11-01

Status: accepted for the ordinary-root/managed-game-array profile. This is not
entity/thread/world or complete GSC support.

## Problem and evidence

Assessment 0082 guards native root initialization, primary/compiler/VM dispatch,
value/tag/ref tails and current-thread receiver lookup with 146 rejected mutations.
Level and anim are ordinary object roles; game is a value cell initialized with an
array; self belongs to the thread receiver. Resolving them as locals, embedding
host IDs in code or fabricating an offline world would change those distinctions.
ADR 0081 now supplies the managed ownership needed to read a host game-array value.
Ordinary/nested logical lifetimes and full source arrays remain separate gates.

## Decision

Add the semantic KernelScriptRoot categories and a typed LoadScriptRoot
instruction. It pushes one checked value and carries no heap identity, native
opcode number, serialized ABI or capacity. Bound module/package compilation lowers
the existing four primary forms; restricted standalone source profiles remain
unchanged. Standalone typed programs can execute the instruction with a context.
Field reads/writes on supported ordinary roots use existing checked field lowering
and source maps. No separate implicit local namespace is created.

KernelScriptRoots is a redacted, Copy, fixed binding view constructed explicitly
by the host from three ordinary ObjectIds (level, receiver, animation) and one
managed ArrayValueId for game. Its constructor does not allocate, validate or
create owners. KernelHeapContext::with_script_roots installs it alongside the
existing borrowed stores. Each invocation validates every supplied binding before
entry cloning, including bindings unused by its selected path. Binding IDs remain
fixed through ordinary direct/indirect/package calls; no ambient world resolver,
implicit always-level self, host callback or new dependency/crate is introduced.

The context includes ordinary root graphs in every pressure root inventory before
collection, even before their first source read. Duplicate ordinary bindings are
valid and remain shared identities. Game's live host location already roots its
data through managed storage. A game read clones a distinct VM-owned location
through ADR 0081, including pending admission and prepaid cleanup. It never inserts
a raw borrowed host handle as an owning cell. Top-level game results transfer one
owner to the host, requiring explicit release. Copying the view, values or outcomes
in Rust does not create extra managed owners.

Validation, reads, copies, root discovery, collection and ownership metadata share
global heap work and simultaneous cache/registry/root/mark scratch admission.
Only fixed view fields are outside logical scratch, as with other fixed context
fields. Caller-supplied root inventories are checked even when an invocation
returns before reading them. Absent bindings fail only at an executed root
instruction with ScriptRootContextRequired. Skipped accesses require no world.

The normal offline command supplies no root view. It reports an honestly located
runtime failure for executed roots, using existing private numeric source locations
and JSON/readable logging. It does not initialize placeholder level/self/game/anim
state. The separate script_roots example explicitly supplies its host objects and
game value before executing the included real source file.

## Scope and consequences

This first view accepts ordinary structs and a managed game array only. It supplies
no entities, changing method receivers, threads, scheduler, notifications, animation
assets/#animtree, automatic loader or map/server lifecycle policy. A game binding
remains an owning value location; a nonowning ArrayId snapshot cannot replace it
for future writable COW selection. Implicit game replacement and general source
array construction/indexing/size/comparison/stores remain unsupported. Ordinary
root fields still reject managed array values, preserving 0071/0075 lifetime gates.

Ordinary objects keep the existing explicit-root shared-storage model. This change
does not implement native object RC, cycle-retained aliases or native thread
creation/cleanup. A host can choose level as receiver explicitly; it must not be
silently substituted for a different receiver. The closed binding view deliberately
avoids a speculative generic host-services framework. Later methods/entities and
game lifecycle need their own context/lifetime decisions and acceptance evidence.

## Validation

Fourteen core integration tests and two process tests cover all four roots,
distinct/shared bindings, field updates, direct/indirect/qualified/imported package
calls, image reuse with new hosts, standalone execution, host return transfer,
missing/stale/foreign/absent-object inventories, unsupported game operations and
source/log privacy. Exact budgets and every shorter instruction/setup/heap-work
prefix preserve host owners; pressure prefixes combine cached literals, VM-owned
game copies and otherwise root-only ordinary/string graphs. Invalid supplied roots
reject before mutation or entry copying even when the source never reads them.

Wide execution writes 70,000 root fields without implicit local slots. Churn reads
game 100,000 times under a 1024-byte scratch ceiling with two managed value slots.
A first game read at 12,001 frames tests lazy registry admission and both successful
return and an exhausted final-return budget, without recursive host destruction.
Source array syntax/operations and method/thread rejections retain their gates.

Run `cargo run -p cod4x-script --example script_roots --locked`. The example uses
examples/gsc/roots.gsc and returns 61: two functions, 26 VM steps, two peak frames,
227 admitted heap-work units, 544 peak context scratch bytes, three fields, one
remaining host game owner and four retained value slots. These are local logical
measurements, not native performance, RSS or joinability claims.

Full Windows formatting, all-target denied-warning Clippy, debug/release workspace
tests, denied-warning rustdoc, policy (941 sources/eight crates/largest 303 lines)
and diff checks pass. Script suites contain 323 frontend/236 legacy tests, 15
private units and two doctests; both new process tests pass. Linux/hosted CI is
unrun. The native guard still rejects 146 mutations and structural checks preserve
99 replays. No native whole-compiler/VM/ABI/lifetime execution is added. Full GSC,
S5, stock/mod acceptance and playable-server gates remain open.
