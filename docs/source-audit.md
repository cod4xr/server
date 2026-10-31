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

## Open research

Inventory all public protocol variants, authentication dependencies, third-party
components, assembly-owned behavior, content formats, native plugins, GSC quirks,
and original float behavior. Build reproducible reference traces before changing
the baseline. Do not copy operator configuration or game content into this tree.
