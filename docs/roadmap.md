# Rewrite milestones

Each milestone requires implemented behavior and recorded evidence. Documentation,
scaffolding, mocked responses, and passing local unit tests do not replace its
acceptance gate. The server rewrite remains incomplete until S7 passes.

| Milestone | Scope | Acceptance gate | State |
| --- | --- | --- | --- |
| S0 | Repository standards, pinned source, primitive codec | Local checks and independent byte vectors | Initial implementation |
| S1 | Full packet/message codecs and transport | Cross-peer packet traces; loss/reorder tests; malformed-input fuzzing | Envelope/command/info codecs and query UDP implemented; gate open |
| S2 | Configuration, commands, challenge/auth lifecycle | Unmodified client reaches authenticated connection; bounded state | Open |
| S3 | Filesystem, maps, assets, checksums, downloads | Stock/custom content loads; pure checks and archive-path tests | Encoded catalog/IWD, BSP containers, fastfile envelopes/root metadata implemented; gate open |
| S4 | Shared simulation, collision, movement, weapon/entity state | Replayed inputs match reference state and prediction behavior | Open |
| S5 | GSC compiler/VM, builtins, game services | Stock modes and named existing GSC mods run correctly | Open |
| S6 | Integrated snapshots and playable server | Unmodified clients play full sessions on stock/custom maps | Open |
| S7 | Operational parity and release | Compatibility matrix, soak/load/fuzz/security/performance gates pass | Open |
| C0 | Custom client planning and version inventory | Named server release matrix and client architecture ADRs | Open |
| C1 | Custom client networking/content/simulation | Joins and plays against each supported ordinary CoD4X server | Future |
| C2 | Client presentation, input, audio, prediction polish | Functional and performance acceptance on target platforms | Future |
| X0 | ApX extension protocol and host interfaces | Capability negotiation and unmodified-peer fallback conformance | Future |
| X1 | ApX-specific server/client behavior | Product-specific acceptance using stable engine interfaces | Future |

Work can move between milestones when dependencies require it; this is a map of
acceptance gates, not a claim that complete network joins precede asset loading.
Client-aware API design starts now; client product development follows the server
and shared-core compatibility work. GSC mod support and native plugin support
must remain separately tracked.

## Next concrete work

1. Continue remaining delta/user-command/snapshot codecs and legacy/profile
   inventory above implemented channel, Huffman, XOR, and index primitives.
2. Add coverage-guided parser fuzzing and deterministic impairment workloads.
3. Extend the query-only runtime with verified configuration, explicit lifecycle
   ownership, and implemented channel services. Preserve structured logging and
   aggregate overload metrics. Do not pretend to load maps or admit players while
   simulation and assets are absent.
4. Implement archive mount order, fastfile/map readers, pure admission, and
   validated imports above the encoded catalog/IWD layer. Build reference
   join/gameplay traces and asset/scripting inventories. Resolve
   architecture and ABI decisions in ADRs before widening the implementation.

## Release evidence

Record OS/toolchain/source revision, actual peer versions, content/mod hashes,
test workloads, resource ceilings, artifacts, and outcomes. Keep an explicit list
of unsupported cases. Compare server frame time, jitter, packet loss recovery,
CPU use, memory growth, and long-session stability on repeatable workloads.
No "all servers", "drop-in replacement", or "production ready" claim is valid
until the matching matrix and release gates pass.
