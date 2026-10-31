# Initial source audit

Pinned repository: https://github.com/callofduty4x/CoD4x_Server

Revision: `ddd65fce1a1093017bad2dbb6840d75d11c0d56f`, committed 2026-09-18.
The reference checkout is separate from the Rust tree. This is a subsystem-level
audit, not a claim that all upstream routines have been understood or ported.

The top-level `src` directory contains 83 C files, 45 C++ files, and 89 headers.
`src/asmsource` contains 110 assembly files. These figures exclude nested asset,
platform, third-party, and tool sources. The makefile builds C, C++, and NASM and
selects 32-bit/SSE2 compilation. Large parts of game and engine behavior reside
in assembly and must be researched and replaced rather than silently omitted.

| Responsibility | Reference entry points | Rewrite evidence required |
| --- | --- | --- |
| Platform/bootstrap | `sys_main.c`, `unix/`, `win32/`, `sys_thread.c` | Initialization, timing, shutdown, filesystem |
| Configuration/commands | `cvar.c`, `cmd.c`, `qcommon_parsecmdline.c` | Flag semantics, config precedence, command parsing |
| Datagram/channel | `sys_net.c`, `netchan.c`, `netchan.h` | Framing, qport, fragmentation, sequence behavior |
| Message formats | `msg.c`, `msg.h`, `huffman.c`, `net_game_conf.h` | Primitive and full packet differential tests |
| Connections/auth | `sv_main.c`, `sv_client.c`, `sv_auth.c` | Actual handshake and join against named clients |
| Server lifecycle | `sv_init_mp.cpp`, `sv_main.c`, `sv_game.c` | Spawn, map rotation, disconnect, restart |
| Snapshots | `sv_snapshot.c`, message delta routines | Matching baseline/delta traces |
| Simulation/movement | `asmsource/_bg_pmove.asm`, `bg_jump.cpp` | Input/state trace parity and prediction |
| Collision/physics | `cm_*`, `asmsource/_cm_*`, `asmsource/_phys_*` | Trace/collision correctness and limits |
| Game rules/entities | `g_*`, related assembly | State transitions and stock game modes |
| GSC compiler/VM | `cscr_*`, `asmsource/_scr_*` | Language, VM, scheduling, builtin conformance |
| Assets/archives | `db_load.cpp`, `xassets/`, `filesystem.c`, `zlib/` | Validated content loading and integrity |
| Plugins/extensions | `plugin_handler.c`, `plugins/` | ABI decision, isolation, attribution |
| Operations | Server command/log/download/master routines | Observability, auth boundaries, soak/load tests |

Some responsibilities span multiple files and assembly routines. Source names
are research entry points, not proposed Rust module boundaries. Further audit
must expand and correct ownership before implementing each subsystem.

## Completed investigation

`MSG_WriteByte`, `MSG_WriteShort`, `MSG_WriteLong`, `MSG_WriteInt64`,
`MSG_WriteFloat`, `MSG_WriteBits`, their read equivalents, and split-buffer access
were inspected. The independent byte/bit cursors are preserved. Hash verification
uses the canonical Git blob after normalizing checkout CRLF to LF.

`SVC_Info`, `SVC_Status`, connectionless dispatch, channel header definitions,
`Netchan_Process`, and version definitions were inspected. A deliberately limited
query-only service implements response framing and independent stricter budgets.
It is not a port of the complete server's mutable cvar/player metadata behavior.
Original transmission, tokenization, and info functions produce differential
fixtures. Ordinary channel state is implemented with documented failure changes.

The static Huffman tree/functions and ordered entity/client index routines are
ported and checked against generated original C fixtures. Final compressed-code
overread is replaced by bounded virtual zero padding. Asset pool defaults and
overrides, configstring ranges, packed script structures, and collision indices
were inspected for docs/limits.md; their expanded replacements remain open.

`filesystem.c` path comparison, archive-local chain order, ZIP positioning, and
seeded CRC32 checksums now ground the IWD reader. `player.h` command fields,
`q_math.h` angle conversion, and the keyed command decoder/helpers in `msg.c`
are pinned and independently exercised. Decoder types use peer-sized values;
they do not impose those widths on future simulation or asset storage.

`com_bsp_load_obj.c` supplies borrowed-container layouts and original lookup/count/
checksum fixtures. `db_load.cpp` supplies unsigned v5 envelope/header/inflate rules,
root loading order, logical stream alignment, direct offsets, alias indirection,
temporary-block rewind, and deferred stream behavior. `xassets.h` supplies 32-bit
root/record layouts. Independent synthetic envelopes/root tables and read-only stock
samples validate container/prefix behavior; original C block conversion supplies
ninety valid relative-offset cases. These do not establish full relocation or graph
validation. The source's native pointer narrowing never enters the Rust runtime.

The actual numeric truth/negation/complement helpers and VM_CalcWaitTime now supply
827 semantic cases. f2rint uses SSE control-state rounding; the oracle explicitly
selects nearest with FTZ/DAZ disabled, while Windows bootstrap requests nearest.
The script kernel preserves defined scalar/wait behavior and rejects undefined
wait conversions. Heap/reference cleanup, error unwinding, scheduling, and parser/
bytecode execution remain separate work. _scr_yacc.asm contains lexer/parser
tables/actions; no editable source grammar was found in the pinned tree.
The loader in tools/cod4_dasm/src/_scr_main.asm supplies the plus entry selector;
it is now hash-pinned. Structural LALR data from _scr_yacc.asm is recovered with
signed widths and checked padding. The star entry accepts expressions/statements,
not only expressions. Table replays and the bounded Rust tree builder do not
execute native semantic actions or prove compiler/stock-mod conformance.
SL_ConvertToLowercase now supplies 125 original ASCII/C-locale conversion cases
with stubbed string/ref services. Its >8191-byte name fallback is identity behavior,
not a Rust storage cap. Hash-pinned _scr_compiler.asm and cscr_variable.h ground
local duplicate/developer-region handling and the native developer_script flag.
Mechanical branch/profile checks do not execute the native compiler; import/call/
parameter/local binding, bytecode, and complete legacy semantics remain open.
Original scalar helpers now supply 12,416 coercion/comparison/division cases under
an explicit nearest/SSE profile. Scr_CastWeakerStringPair and Scr_EvalPlus also
ground 54 valid plain-string results and 54 intentional scratch-limit expansions.
Their string/ref services are stubbed and mixed-kind services abort. Neither
oracle establishes full VM reference cleanup, error unwinding, or execution.
Hash-pinned tools/cod4_dasm/src/_scr_variable.asm now grounds an explicit integer
wrapping/shift profile. Exact instruction-block checks and isolated x86 instruction
cases supplement defined-domain original C arithmetic without executing undefined
C signed operations or full archival assembly routines. Native MIN%-1 IDIV would
fault; the safe Rust helper returns a typed error. See ADR 0022.
Grammar action constructors/tags, expression/primitive nonterminals, and scr_vm.h
operator identities now guard typed operator views. Tests inspect source operand
order/precedence without replaying native semantic actions. This local shape layer
does not validate primary forms or execute them; see ADR 0023.
EmitOrEvalExpression also exposes the native 32-value compilation stack and short
expression-branch patches. These are compiler capacity-removal targets, not limits
on new engine bytecode. Primary research distinguishes signed literal nodes,
parenthesized lists, calls/references, #animtree, and breakon forms; none is executed
by the operator view. Runtime/debugger availability needs separate policy/evidence.
All 22 primary production/action profiles now guard separate borrowed primary
views. Raw sign/token identity, grouped lists, calls/references, special objects,
size/empty-array/animation/#animtree/breakon forms stay distinct. No original
semantic actions execute, and descendants/runtime modes remain unresolved.
Expression-list rules 111/112 prepend native nodes, 113 aliases the nonempty list,
and 114 creates the empty node. Their checked profiles ground iterative flat
source-order indices. Native evaluation/push order is not inferred from that index;
arity, call/parameter semantics, and executable capacities remain separate work.
Rules 28..39 now guard module/name/reference/callee/mode/invocation roles through
lengths, nonterminals, and semantic constructors. Their native lowercase actions
remain source evidence; borrowed Rust views retain exact spelling for later binding.
Thread/method/indirect syntax creates no runtime action or target resolution.
Rules 62..66 guard named/field/index/dollar/reserved-self reference roles. Targeted
parser-transition research and source-table cases distinguish object.self from
object.Self; a broader all-production recovery exceeded its research work cap and
is not claimed complete. Reference views do not infer variable/object semantics,
mutability, or runtime authority from those structural roles.
Rules 68/69 identify value/void returns, 94/97/108 preserve statement wrappers,
and 109/110 append/end body lists. Their pinned length/nonterminal/action profiles
now guard a restricted scalar source compiler. That compiler supports actual
owned numeric code/maps and lazy branches; these source checks still do not run
the original compiler. Negative integer MIN admission is an explicit checked
domain policy; undefined native atoi overflow is not used as evidence.
Rules 67..110 now guard direct statement, wrapper, optional update, and body-list
profiles, including the ten compound operator opcode identities. Typed views
preserve event lists 119..124 as opaque distinct payloads, not ordinary argument
lists. Source-table research finds the bare-semicolon statement outside the
nonterminal used for a for-loop body; valid empty-body tests therefore use braces.
These are pinned structural/action profiles, not native compiler/VM execution.
Scr_CalcLocalVarsVariableExpressionRef compares the registration count against
0x3f; Scr_FindLocalVarIndex reads/sets initialization bits and reports an
uninitialised-variable compiler error on invalid reads. The scalar backend now
uses separate initialized-state/value slots and wide checked indices with budgets,
without claiming source definite-assignment equivalence. Its exact internal
argument binding does not infer native missing/extra-argument behavior from the
EmitThreadInternal parameter walk. That adapter and source binding remain open.

## Open research

Inventory all public protocol variants, authentication dependencies, third-party
components, assembly-owned behavior, content formats, native plugins, GSC quirks,
and original float behavior. Build reproducible reference traces before changing
the baseline. Do not copy operator configuration or game content into this tree.
