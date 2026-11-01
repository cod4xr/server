# ADR 0095: Source assignment through initialized local array cells

Date: 2026-11-01

Status: accepted for initialized-local source writes on Windows. General nested ownership,
complete GSC and joinable gameplay remain open.

## Source behavior

Bound modules and supplied packages now compile plain indexed assignment through
an initialized local name, using ADR 0088's existing StoreArrayLocal and
ClearArrayLocal instructions. ADR 0094 supplies the private-local lifetime rules
that determine whether later mutation must copy shared array data. One small
compiler leaf selects this profile; no instruction, crate or dependency is added.

Normal assignment evaluates RHS, then key, then borrows the frame's existing
owning location. It never loads/clones the receiver to obtain a writable value.
Undefined locations autovivify, and shared locations select COW before key
validation. A direct literal Undefined selects ClearArrayLocal instead. Grouped
or called Undefined takes the normal lvalue path, including temporary child
selection/removal and its field/key quotas. The literal clear still validates
encoded literal admission. Store/clear source maps identify the opening bracket;
failures inside operand calls retain their own function/source location.

Plain assignment and literal clear preserve remaining entries. COW reverses
existing sibling order while ordinary object payloads retain identity. Supported
payload categories are the existing checked nonarray values, including runtime
strings, ordinary structs and image-bound function tokens. Independent populated
arrays can cross calls, includes and returns with explicit transferred host
ownership. Reads push result graphs into a frame before the last receiver's
release can be followed by collection.

## Boundaries

Only an existing definitely initialized plain local target is admitted. Explicit
Undefined and missing parameters satisfy initialization; a never-bound name or a
cleared private binding does not. Nested/field/root/game array targets and indexed
compound/postfix updates remain rejected at compilation. Stored Array values
remain rejected at execution before publishing an outgoing owner edge. This
closed graph is the lifetime argument of ADR 0088, not general logical RC/cycle
or scheduler ownership. Ordinary field assignment retains its existing profile.

Standalone scalar/primitive source validation gains no mutable array authority.
Legacy source keys retain the named strict key conversion profile; wide internal
storage is separate. Complete string keys are preserved, with the private tagged
encoding included in key-byte admission. No short local/field/key/owner counts are
introduced. Existing developer, namespace, image, call and operand policies apply.

The earlier ADR 0087 gate regression retains its inventory identity. Its former
plain-local write input is intentionally replaced with a nested indexed target;
the other compound/postfix/game cases remain rejected. New source tests prove
that initialized-local writes activate, while never-initialized targets and
nested owner edges stay closed. This is an explicit profile expansion, not an
unchanged claim that every old rejection outcome still applies.

## Acceptance evidence

Twenty-two new frontend tests cover actual source writes, autovivification,
literal/grouped/called Undefined, shared object/string/function payloads,
indirect calls, indexed for clauses/continue and rejected targets. Fifteen distinct control-flow programs
observe native sibling order after mutation, exposing missing or extra aliases
across branches, loops and switch exits. Source RHS/key effects are ordered, and
invalid keys observe a real detached copy before failure. Repeated failed and
successful writes share one context without retaining VM owners or mutating host
payloads.

Compiler work is tested at every shorter prefix and all exact/one-less resources.
Runtime COW, child creation, call setup, instructions and cleanup are tested at
every shorter work prefix and sixteen one-less storage/frame/scratch resources.
Computed string-key deletion admits its complete temporary tagged key, including
the tag byte. COW/insertion pressure preserves RHS graphs held only on the
operand stack. Independent runtime-string/shared-object read results survive
the last array owner and subsequent callee collection without host or full-string
literal-cache backing; exact and exhausted combined root/mark scopes are checked.

Source stress covers 70,000 entries, a 70,000-byte key, 100,000 writes to a single
entry alongside private aliases with 4 KiB scratch beside 70,001 unrelated host owners, and first
populated-array construction 12,001 frames deep with an exhausted return. The
array_writes source example returns integer 42. Three process tests cover actual
offline execution, populated include-package results/readable logs, and private
located failure after COW with zero live VM owners.

Required acceptance includes full Windows formatting, all-target denied-warning
Clippy, debug/release workspace tests, denied-warning rustdoc, preserved earlier
test inventories, 306 selected native array traces at O0/O2, 141/61 write/lifetime
mutations, 99 structural replays, scope evidence, suite registration and repository
policies. Native helper fixtures remain unchanged. These execute selected helpers,
not a complete native parser/compiler/VM/freeing path. Linux and hosted CI remain
unrun. Full arrays, entity/engine builtins, threads/waits/notifications, expanded
language services and stock-mod acceptance remain future work.

All required Windows gates pass. Complete debug/release workspace runs execute
389 frontend/257 legacy tests, fifteen private script tests and two script
doctests. After the final for-clause regression, the complete current frontend
suite passes all 390 tests in both profiles. Earlier test names remain preserved.
All-target Clippy and rustdoc deny warnings; formatting, native/structural/scope
replays, 35 scope admissions, twelve suite-policy regressions, policy and diff
checks pass. Policy checks 1054 sources/eight crates/largest 287 lines.
