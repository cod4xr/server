# Logging and operational contract

The process owns logging installation. Libraries emit structured events without
changing the global subscriber. `cod4x-observability` owns formatting, sink setup,
filter validation, workers, and dropped-record accounting. It has no dependency
on server, client, game rules, or protocol internals.

## Configuration and output

Development builds default to compact readable stderr at `info`: elapsed time,
colored severity, message, and key/value fields. Release builds default to JSON.
`--human-logs` and `--json-logs` explicitly select either representation in both
build types. Files always remain JSON. Human timestamps show monotonic elapsed
time since logging startup; JSON timestamps retain absolute UTC for correlation.

`--color auto|always|never` controls readable console colors. Auto enables ANSI
only for a terminal and respects `NO_COLOR` and `TERM=dumb`; redirected output is
escape-free. JSON never receives ANSI escapes, even when color is forced.
`--log-level` accepts explicit tracing filter directives; no environment variable
silently overrides it, and regex matching is disabled. Invalid level directives
fail startup.

`--log-dir PATH` enables newline-delimited JSON files. This must be a dedicated
application log directory per server instance. The rolling appender uses UTC daily rotation, the
`cod4x` prefix, the `jsonl` suffix, and retains at most 14 matching daily files by
default. Existing files with that naming pattern participate in retention.
Unrelated files must not be placed under application ownership. Rotation bounds
file count, not bytes per file or total disk consumption; disk quotas/collection
remain an operational concern until a size-budgeted sink is implemented.

Each sink has a 4096-record queue by default. Full queues drop records rather than
blocking the game/network thread. Queue limits are validated, and drop counters
are included in periodic/final metrics. This counts queue overflow, not every
downstream disk/collector error. The runtime retains worker guards and flushes at
normal shutdown with the appender's bounded timeout. SIGKILL, machine failure,
and abrupt termination cannot guarantee delivery. Logs are not durable storage.

## Events and levels

| Event | Level | Fields |
| --- | --- | --- |
| `server.started` | info | build version, bound local address, explicit runtime mode |
| `server.metrics` | info | uptime, received/replied/dropped counts, send errors, dropped logs |
| `server.stopped` | info | shutdown reason, dropped log count |
| `server.failed` | error | typed local I/O failure category |
| `content.started` | info | offline IWD verification or fastfile inspection mode |
| `content.verified` | info | entry/file counts, encoded/metadata/declared bytes, legacy checksum |
| `content.envelope_verified` | info | encoded/payload/external/total-block bytes; graph remains unvalidated |
| `content.prefix_inspected` | info | root slot/record counts, prefix/remaining/virtual bytes; bodies stay unvalidated |
| `content.rawfiles_inspected` | info | root/present slots, inline name/buffer bytes, virtual/temp usage, logical peak metadata |
| `content.failed` | error | sanitized content failure or local I/O category |
| `script.started` | info | explicit offline legacy structural inspection mode |
| `script.syntax_inspected` | info | source/tree/declaration counts, scan/grammar/declaration work, separate logical parser/index/declaration metadata |
| `script.failed` | error | read/index/parse/declarations/binding phase, fixed category, current/previous source locations |

Timestamps, levels, targets, and active span context are included in JSON records.
Metrics are emitted every ten seconds and at normal shutdown. Rejected traffic
is counted by low-cardinality category; it does not generate one log event per
packet. Persistent errors must be summarized or rate limited rather than logged
in an unbounded retry loop.

Handshake-probe metrics add `probe_challenges` and `probe_verified_rejections`.
These count prepared diagnostic replies, while `replied` counts successful sends.
No players are admitted, and a verified cookie does not authenticate an account.
Nonce, GUID, key, password, and userinfo values never enter probe logs.

Future connection events will use process-generated connection IDs/spans. Do not
use player-controlled strings as metric labels. Authentication decisions, map
lifecycle, script failures, content validation, and extension negotiation need
their own stable event names as those services are implemented.

## Sensitive and untrusted input

Never attach packet bodies, RCON/password arguments, authentication tokens,
challenge echoes, complete configurations, or raw command lines to events, even
at trace level. Current process tests send a recognizable secret through real
UDP queries and assert it never appears in captured logs. Protocol error types
and query drop categories contain no received contents. Ordinary discovery logs
also omit remote addresses; the startup address is the local listening socket.
Offline content verification excludes host paths, archive entry names, and
payloads from logs. Full verification includes overridden original entries;
counts/checksums are metadata, not a claim of authenticated content.
Fastfile inspection reports envelope integrity only and never emits an asset-ready
or server-started event. Snapshot failures log a fixed category; stderr omits host
paths and arbitrary operating-system error strings while retaining the I/O kind.
Zone-prefix inspection is another explicit offline mode. It logs root counts and
byte accounting, never string contents, type/header arrays, or unvalidated bodies.
Raw-file-only inspection reports typed decode accounting. Present slots include
aliases, so they are not a unique-file count. Metadata is logical peak decode
storage, not retained memory or RSS. The command does not register assets or run
scripts, and logs omit names, payloads, and host paths.
Offline GSC inspection uses the same bounded file-snapshot boundary and reports
structural success only. It emits no runtime-started or script-executed event.
Parse locations use original source bytes, one-based lines/byte columns, and no
hidden selector offset. Raw source, names, host paths, and arbitrary OS error
strings remain outside logs/stderr. Scanner/index/parser budgets fail explicitly.
Local binding adds explicit developer mode, total/active/ordinary/developer function
counts, borrowed name bytes, and separate logical index storage/work. Duplicate
diagnostics expose both source offsets/line/byte columns without function names.
This is local-definition inspection, not full symbol/call linking or execution.

Any future logging of user-controlled fields requires explicit size limits and
escaping. Use typed outcome/error categories rather than printing peer-provided
error text. Keep actual I/O/configuration causes available to local operators
without exposing credentials. Local initialization failures also return a
nonzero process exit status and an actionable stderr diagnostic.

## Current evidence and limits

Windows tests verify JSON parsing, filter behavior, active connection span fields,
UTC timestamp shape, normal file flush, process lifecycle events, counter values,
and exclusion of actual wire secrets. Linux and release checks for this expanded
runtime are tracked separately in the validation record. Per-file byte rotation,
external log shipping, durable audit trails, and live filter reload are not yet
implemented.
