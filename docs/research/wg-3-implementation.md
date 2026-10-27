# WG-3 implementation evidence

27 October 2026. Implemented in isolated `feat/work-sources` from integration
`dd7c744`, using the approved executable/external-file and public Console seams.
No live project instructions, native runtime messages, remote writes, publishing
or installation were performed. Native recordings remain read-only.

The concrete command/schema contract for WG-4 and other consumers is
[wg-3-source-contract.md](wg-3-source-contract.md).

## Behavior and integration

- `src/tracking.rs` owns local opt-in/bindings, explicit table/ticket adapters,
  bounded last-good/retired caches, exact native auto-binding and compact status.
  Enable/status/disable/sync share provider/session/project/state-root flags.
- `Store::read` adds cached `source_tasks`, `source_snapshot` and
  `source_diagnostics` to the compatible version-1 WorkRecord. Existing explicit
  report, goal/evidence, draft and native-plan schemas remain valid.
- The existing Console refresh path runs source refresh on the Inventory worker;
  it polls every five seconds, with per-source stat caching and per-goal source
  versions. Render/input use cached records. No watch daemon, transcript-content
  parser outside `provider/`, model or network request is introduced.
- `source_snapshot` chooses one canonical effective plan after any successful
  source observation, including an empty snapshot. Explicit reports/native plans
  remain preserved fallback/history. Initially invalid sources use that fallback;
  later errors retain the stale canonical source view. WG-1 consumes the separate
  source projection with optional native owner and truthful raw human labels.
- Supplemental reports match the emitted namespaced task ID exactly, never a
  label. Per-field source authority survives clearing and invalid edits. Source
  status cannot verify outcomes. Existing criterion evidence remains separate.
- Ticket-file dependencies resolve only unambiguous declared IDs in the same
  explicit goal binding. Cycles/self/missing refs remain graph relationships.
  Unknown owner/status/dependency values remain unknown or nonsteerable.
- Files are explicit relative Markdown allowlist entries under the canonical
  selected root. Bounds cover global source count, file/cell sizes, current tasks,
  dependencies, edges, cache/history, registry and compact output. Symlink
  escapes are rejected. Disabled state stops reads and survives restart.
- `--session auto` checks both exact Codex runtime IDs, their own bounded public
  headers and complete parent chain. It does not infer identity from cwd/title/
  freshness. Header-schema knowledge stays inside `src/provider/`.
- Final source-owner review found that legacy explicit reports allow direct
  native IDs without metadata proof. Source entries now require WG-2's added
  identity-proof entries before selecting/steering a direct native owner. The
  source-only guard preserves that legacy report interface, preserves the raw
  source owner label and canonicalizes an exact native root ID to `main`.

## Vertical RED/GREEN record

Commands ran with `CARGO_TARGET_DIR=/tmp/zoetrope-wg-build/sources` and `--locked`.
Each row used `cargo test --test tracking FILTER -- --nocapture` before and after
the associated minimal implementation. No parser/private-helper tests were added.

| Filter | RED observation | GREEN behavior |
| --- | --- | --- |
| `exact_binding` | tracking command parsed as a second target argument | idempotent exact scope, persisted disable, no instruction edit |
| `designated_table` | public record had no source tasks | selected table, escaped pipes, human/native owners, stable IDs and valid removal |
| `ordinary_file` | normal document edit left cached title unchanged | existing worker refresh observes edits; disable stops refresh |
| `explicit_ticket` | unsupported ticket selector was stale | bounded explicit metadata imports, body/fenced examples ignored |
| `source_owned` | missing context was not supplemented | exact-ID context supplements; source-owned fields win and clear |
| `automatic_identity` | literal auto session was retained | exact root/current-thread headers and ancestry validated; mismatch rejected |
| `multiple_goal` | project opt-in remained true after disable | exact source scopes isolated; project disable stops all bindings |
| `canonical_sources` | old report remained alongside canonical plan | effective context chooses canonical source and retains old fallback/history |
| `source_owned` (invalid schema extension) | invalid source claimed Next authority and hid last-good report context | values and field authority commit atomically only after valid parsing |
| `bounded_refresh` | second source exceeded goal-wide population | aggregate bounded source set rejected without replacement |
| `reenable` | missing allowlisted file prevented re-enable | exact binding resumes with stale last-good tasks |
| `ticket_set` | ticket dependency pointed only into its own file namespace | unambiguous explicit ID resolves within the bound ticket set |
| `source_set_growth` | aggregate overflow failed refresh rather than marking stale | last-good source retained with visible stale diagnostic |
| `invalid_missing` (single pipe) | malformed row panicked the executable | malformed row reports stale without losing last-good data |
| `fenced_task` | nested unlike fence exposed example tasks | matching fence marker/length keeps examples non-authoritative |
| `malformed_native_owner` | native:main became a coordinator reference | malformed native/alias tokens stay unavailable |
| Fleet `source_native_owner` | foreign-root source owner exposed live Guidance/Queue/Stop; proof-unavailable recovery stayed unavailable | absent/foreign identities cannot receive controls; valid parent metadata recovery selects the exact child; native root ID selects its canonical coordinator |

Regression fixtures additionally cover initially invalid fallback, missing/
malformed/duplicate/oversized sources, recovery, valid empty/removal, unchanged
content without persistent writes, symlink escape rejection, restart, per-field
provenance, unknown/human/unassigned ownership and no automatic verification.

## Validation and representative resources

- `cargo fmt --check` and `git diff --check`: passed.
- `cargo clippy --locked --all-targets -- -D warnings`: passed.
- `cargo test --locked`: passed again after merging integration `55d35bb`
  (including 59 Fleet UI fixtures plus the 15 tracking fixtures).
- `cargo test --locked --test tracking`: 16 public tracking fixtures passed after
  the final native-owner safety change.
- `cargo test --locked --test fleet_ui source_`: targeted source graph/owner/
  runtime-peer checks passed after merging the WG-2 identity-proof follow-up
  `d1d6344`. The peer rejects every steering message in the unavailable-owner
  fixture; current-scope metadata recovery and exact root normalization are
  observed without sending any messages. Root owns the final full combined run.
- `cargo test --locked --no-default-features`: passed (176 portable unit tests and
  10 public Console tests; native-only suites correctly contain no tests).
- `cargo +1.88.0 check --locked --all-targets`: passed.
- `cargo clippy --locked --no-default-features --all-targets -- -D warnings`:
  passed after the integration merge.
- Browser/frontend integration verification remains WG-6; native-only additions
  are feature-gated and portable tests preserve replay compatibility.

A disposable fixture used one 36,230-byte task table with 512 rows and a
393,302-byte local tracking cache. A temporary executable probe called the public
Console refresh path with real files: initial load 37.7–40.1 ms; 100 unchanged
refreshes 659–666 ms (~6.7 ms each) in an unoptimized debug build. GNU time reported
11,024 KiB peak RSS, zero filesystem inputs/outputs during that probe, no swaps.
At the normal five-second inventory interval this is modest local worker work;
it is not a whole-application UI benchmark. The temporary probe source was removed.
Twenty public status processes took 356 ms total; twenty sync processes took
316 ms total; each status response was 340 bytes. A default 16-task context page
was 13,489 bytes with 496 tasks omitted and `next_work_offset: 16`. These samples
measure this fixture/machine only; configured limits also reject larger inputs
and retain last-good state rather than assuming arbitrary source sizes are safe.
