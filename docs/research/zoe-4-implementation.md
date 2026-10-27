# ZOE4 terminal console implementation

Worktree `/Users/melikhov/Repos/zoetrope-worktrees/zoe-4`, branch `feat/zoe-4-terminal`, initial integration `fa10ab0`, clean. Rust1.88 override and separate target directory `.../zoetrope/target/zoe-4`. Read approved spec, tickets, CLAUDE, TDD and tests/mocking references, codebase-design, architecture/provider seam, and root handoff notes.

Initial exploration only while ZOE2 finalizes; no tests or product changes before dependency integration is confirmed by root.

## Intended interface and implementation

A native FleetConsole module exposes configuration, public crossterm input, cached poll and ratatui draw. Tests interact at that approved seam using actual state files/session fixtures and an external runtime protocol peer. Native filesystem, persistence and discovery live on an inventory worker; control uses an independent bounded worker; explicit editor opens use one isolated in-flight worker. Rendering consumes cached projections only. Bare `zoe` runs this console; old targets/inspect remain intact and `--project` preserves cwd-follow.

Use stable provider/session/agent identities in Rataflow, mutate cards without moving existing nodes on background refresh. H shows same graph history; / search; keyboard panes for feedback, questions, criteria and documents; explicit sends, no default answer or uncertain retry. Target IDs remain internal; cards emphasize meaningful goal/name/activity and evidence-backed progress. Native active/waiting overlay supersedes timestamp fallback.

Provider-owned public activity projection must skip analysis/thinking and must not consume the existing conflated FactKind::Reasoning. Explicit public commentary/typed safe file labels and recorded shell purpose are sufficient without a model.

No product changes yet. Root handles final README/user guide, release installation and real GUI/terminal/performance checks.

## PAUSED WIP checkpoint — user requested save and pause

Dependency integration52a1766 was merged by fast-forward before first test. Product implementation has started, but ZOE4 is incomplete and must not be merged into integration yet.

Current authored files:
- `src/fleet/mod.rs`: initial native FleetConfig/FleetConsole public key/render/poll interface; inventory worker owns Console; cached Rataflow graph; initial evidence progress sidebar; H current/history toggle. Blocking discovery/state is off the UI thread. This is a first-slice implementation, not complete frontend.
- `src/lib.rs`: native-gated fleet module export.
- `tests/fleet_ui.rs`: actual public key/render test and fixture helpers.

### First vertical slice

`CARGO_TARGET_DIR=/Users/melikhov/Repos/zoetrope/target/zoe-4 cargo test --locked --offline --test fleet_ui opening_fleet`

RED: unresolved zoetrope::fleet interface. Initial implementation also exposed overlapping graph nodes when history toggled; explicit reflow now updates positions of existing nodes as well as new nodes. Final run GREEN: 1 test passed (0.08s). It shows evidence-backed1/2 progress and changes to the same Allhistory graph through H.

Last exec sessions56332(baseline check),58365(RED compile),4430(initial failed graph rendering),62025(final GREEN) all completed. No known running build/test sessions remain. No checks started after pause request.

Known WIP warnings: unused Rect import; InventoryView.path and FleetConsole.notify not yet used. They are planned for later interaction/run wiring. No formatting/lint/full regression has been run on WIP.

### Remaining implementation

- Public selection/expansion/search; exact-target persistent feedback with independent bounded control/state workers; questions/drafts/answer/dismiss/expiredfeedback; criteria editing/evidence/reopen/adoptplan/checkpoint preference; documents with isolated in-flight opener; public activity projection skipping analysis/thinking; real runtime status overlay for quiet active/waiting work.
- Native run loop and default CLI launch, --project old cwd-follow, one owned input stream, in-process read-only replay lifecycle with reader/tailer cleanup, event-driven idle draws, narrowterm/help/mouse behavior.
- Continue one RED→GREEN public seam slice at a time. Existing planned keymap was sent to root but is not yet implemented beyond H/q/Ctrl-C.
- Current code's snapshot change comparison serializes compact graph; optimize with structural equality/generation once UI projection settles. Current positions map reflow and cached UI are initial scaffolding.
- Read root's latest docs commit b1931b6 and merge latest feat/goal-console before resuming. Root owns README/architecture/user guide, release/realterminal/performance/PR; ZOE4 owns product/help/tests.

Do not install, release, push or merge this WIP while paused. Keep worktree `/Users/melikhov/Repos/zoetrope-worktrees/zoe-4` and branch `feat/zoe-4-terminal` for resume.

## Resumed on24 October2026

Pause revoked by user through root. Merged integration981de7e before edits; root docs continue separately.

2. `fleet_ui a_searched` RED missing search/feedback; GREEN searched history selection, typing H/q stays text, exact-goal feedback saved and restored through process restart. UI acknowledges matching completed query before presenting Search: results, avoiding selection race. Independent state writer added.
3. `fleet_ui goal_criteria` RED no criteria editor; GREEN plain-text title/outcome editor, explicit attestation/evidence verification, reopen removes current credit and retains history. UI updates selected graph card from committed state to prevent stale progress labels.
4. `fleet_ui an_incomplete` RED999/1 weights showed100% with1/2 verified; GREEN central WorkRecord label uses<100% nearcompletion, numeric incomplete percentage bounded below100. Full bar will also reserve remaining space until every criterion is verified.
5. `fleet_ui guidance_queue` (escalated for local Unix bind) RED missing recipient/control; GREEN actual key/render→independent protocol peer verifies child-only exact-turn Guide, distinct native Queue with stable client ID, and exact Stop. Outbox persisted before any send; control runs independently from discovery/state work. No real model calls or native session mutations in tests.

Current next slice: expired-question partial answer restart/copy-to-feedback (RED being established), then fresh native question replay/submission/resolution, artifacts, public activity, live overlay and terminal lifecycle/defaultCLI/help/narrowterm. Implementation is ongoing, not ready to merge yet.

## PAUSED AGAIN — user explicitly requested SAVE AND PAUSE

Saved current product/test work as **3f5426f** (`wip(ui): save steering and question workflows before pause`) on `feat/zoe-4-terminal`. Worktree `/Users/melikhov/Repos/zoetrope-worktrees/zoe-4` is clean and retained. Earlier saved first slice aca89c4 is an ancestor; integration981de7e was merged before resumed work. Root documentation commits after981de7e have NOT yet been merged into this WIP branch. No install, release, push or source merge into integration was performed.

### Exact stopping point

Eleven public UI slices were individually GREEN when completed. The twelfth/current slice is RED:

`CARGO_TARGET_DIR=/Users/melikhov/Repos/zoetrope/target/zoe-4 cargo test --locked --offline --test fleet_ui transcript_and_live -- --nocapture`

It fails with **Questions1 of2**, demonstrating that a recorded Expired question and its live replay are still duplicated. Last session23836 completed exit101 before pause; all prior sessions completed. No known running build/test/protocol-peer processes remain. The test peer is RAII-killed/waited on panic. No new checks were started after pause.

The RED test also specifies later steps not yet reached: preserve the existing Beta draft when native itemId merges into the recorded source; submit explicitly, observe Resolved, copy that resolved draft to feedback; then a fresh state directory imports the transcript function_call_output answers as resolved history without showing old answered work as Current. Native request fixtures now carry itemId=`native-question-call`.

### Completed resumed slices (individual checks)

6. `fleet_ui an_expired`: RED missing saved-question view; GREEN local expired text, partial answer save/restart, explicit copy into feedback with no delivery.
7. `fleet_ui native_questions`: RED missing Listen action; GREEN explicit live join, real protocol peer question replay, no default selection, deliberate option choice, deliberate submission and observed native resolution. Native answer state is Submitted until resolution; resolution is not attributed to a particular client.
8. `fleet_ui quiet_waiting`: RED timestamp-only filter hid quiet waiting work; GREEN global read-only loaded inventory/status makes quiet Working/Waiting work Current without resume/start. Runtime-only work is shown without inventing ancestry when a local transcript is unavailable.
9. `fleet_ui documents_open`: RED missing document view; GREEN list does not launch, select second exact Unicode/special-character path, isolated launcher receives that path and reports preview requested.
10. `fleet_ui recorded_async`: RED known actual request_user_input_async form lost; GREEN direct structured questions[{title,options?:string[]}] imported with stable call/prompt IDs, exact producer, Expired/unconfirmed state and no native pending IDs. Answer persists restart; strings of JS/code are ignored.
11. `fleet_ui public_commentary`: RED UUID primary label/missing activity; GREEN first recorded human goal, public commentary and explicitly recorded shell command/purpose shown; synthetic analysis/reasoning placeholders and technical tool names never rendered. Uses provider-owned projection, never conflated FactKind::Reasoning.

No full regression, formatting, clippy or rustdoc run on this WIP yet. Some earlier green slices have not been rerun after later source changes. Known WIP warnings: InventoryView.selected/path unused; FleetConsole.notify unused until executable loop wiring. Large functions are still compact source and need cargo fmt when work resumes.

### Current files/interface

- `src/fleet/mod.rs`: native FleetConfig and FleetConsole::{new,poll,handle_event,draw,shutdown}; cached Rataflow graph and panes. The actual binary default is NOT wired to this module yet.
- `src/fleet/editor.rs`: small UTF-8 editor; cursor/backspace/delete/home/end, Ctrl-U clear, Ctrl-A start, Ctrl-E end.
- `src/fleet/state_worker.rs`: independent bounded32-command atomic local state writer; first selected-record load expires stored native pending IDs, local edits/answers saved without renderer IO.
- `src/fleet/control_worker.rs`: independent bounded controller lane; exact Guide/Queue/Stop, explicit Join/Answer, outbox saved before delivery, runtime events persisted, global loaded observation with one background metadata query per iteration. No model/daemon/settings creation. Metadata/selected status cached.
- `src/fleet/document_worker.rs`: isolated single-in-flight editor opener, explicit only. Does not block steering/state or wait for an external GUI process when console is dropped.
- `src/provider/activity.rs`: public typed commentary/file-label/shell-purpose projection. Explicit commentary/final only for Codex; Claude text blocks only, never thinking. Command input is displayed only for supported shell tools; JS code is not interpreted.
- `src/provider/metadata.rs`: bounded bytes feed public activity plus original metadata/work projections; first recorded human goal and safe activity cached.
- `src/provider/work.rs`: adds direct structured request_user_input_async support; existing plan/native question/Markdown FileChange import remains.
- `src/console.rs`: live observation DTOs LiveActivity/LiveThread, Console::set_live_threads; native Active/Waiting visibility and priority over timestamp fallback; local pending questions/unfinished drafts count as attention; runtime-only graph nodes remain clearly unlinked when no transcript exists. Existing positions are preserved on background refresh and new nodes avoid occupied slots.
- `src/control.rs`: ThreadSnapshot derives PartialEq for cache change detection. NativeQuestion still lacks item_id; see next steps.
- `src/workstate.rs`: DTO PartialEq for cached rendering, unfinished weighted progress label cannot claim100% (uses<100% near completion), numeric incomplete ratio bounded below100; accepted/queued exact draft is not counted as an unfinished unsent draft.
- `tests/fleet_ui.rs` and `tests/support/codex_fleet_peer.py`: approved public key/render and real external executable/protocol seams only.

### Implemented keymap so far

- H/h current/history, / search then Enter apply, arrows or j/k navigate, Enter/Space expand, q navigation-mode quit, Ctrl-C quit.
- f feedback editor; F2 Guide, F3 Queue, F4 Stop in graph/feedback; ordinary H/q characters stay text; Esc saves feedback draft.
- c criteria; e plain title/outcome editor; v evidence/attestation; u reopen; F2 commits goal/evidence editor; Esc cancels those edits; PageUp/PageDown detail scrolling.
- Q questions; L explicit native listen/join; Tab prompt; digits choose without submit; e free-text answer; Esc saves partial answer; F2 explicitly submits when current pending IDs are confirmed; F3 currently copies only Expired questions into feedback; x dismisses locally without native cancellation.
- d documents; arrows select, Enter opens the exact selected Markdown document in the configured reader bridge.
- R replay, ?, zoom/help/mouse/progress-bar interaction, p checkpoints, A criteria proposal, a plan adoption, and defaultCLI/--project are NOT implemented yet.

### Next steps on resume

1. Continue current RED slice before any new feature. Root verified from real owned disposable evidence that native request params.itemId equals transcript request_user_input call_id2/2. Add optional item_id to NativeQuestion and populate from params.itemId. Persist live source as `transcript:{producer.agent_id}:{itemId}` when present, preserving the same draft as the historical record. Fallback to native/thread/turn/request only when itemId absent. No fuzzy matching or IDs invented as native pending.
2. Allow explicit copy-as-feedback for both Expired and Resolved questions. Use wording original request no longer pending; do not claim resolved meant answered or reopen terminal state. Store currently accepts Expired only, so backend/UI wording both need adjustment.
3. Import recorded function_call_output JSON objects with answers as source resolutions. Must handle output-before-capture/head-tail gaps without a strict unknown-question update failing a whole import batch. Suggested durable observed-resolution tombstones or equivalent order-independent handling; preserve drafts, avoid false native pending claims. Historical answered questions should not remain Expired attention forever. Existing CaptureQuestion should not clear a confirmed native request_id/turn_id when an unconfirmed transcript copy is reimported.
4. Finish explicit plan adoption and checkpoint preference/criteria proposal. Root requires the explicit request draft to contain the exact goal provider/root-session and correctly shell-quoted actual state root, even when a child is selected. No automatic sends. Ordinary criteria UI remains plain text.
5. Add actual interactive verified-progress bar (reserve at least one unfilled cell when verified<total), mouse/zoom/help and narrow terminal layout, readable outbox/in-flight state, concise attention indication.
6. Wire native executable default fleet + --project legacy cwd-follow. In-process replay must use one owned input stream and properly cancel old tailer/reader on return; existing tui::run leaves its EventStream task alive if naively nested. R read-only replay should return without losing composer/history/selection and must not send live controls.
7. Complete event-driven native run loop: Notify cached changes, no idle60fps, draft debounce500ms only while dirty, independent bounded IO lanes. Root handles actual release/PTY/CPU/RSS verification.
8. Review resource edges: remove unused full selected-record clone from InventoryView; compact summaries + selected state already held elsewhere. Coalesce selected-target/control requests rather than filling a metadata queue during rapid navigation. Ensure live-only goal state refreshes from disk and no raw UUID remains a primary label.
9. Read root review observations at `docs/research/zoetrope-root-review-2026-10-24.md`; Store::read projects in-flight Sending to Unknown, so UI keeps its own control_pending state while request is in progress. Final backend reviewer/fix agent will audit combined branch.
10. Merge latest integration before final handoff, format and run scoped/full checks (Unix peer tests need escalation in this sandbox); leave install/release/push/PR/docs to root.

Paused means saving only until user resumes. Keep WIP branch/worktree/cache; do not merge this checkpoint into integration.

## Resumed25 October — stable library checkpoint

Merged integration70e6362. Completed existing RED question identity journey: optional native item_id retained and mapped to same transcript producer/call source; partial Beta answer preserved without duplicate question; terminal Resolved allows explicit feedback; recorded output answers imported via order-independent resolution map. Empty/unattributed native resolution stays Resolved attention; explicit nonempty recorded answers become answered history. Unconfirmed transcript reimport cannot erase confirmed native IDs. Existing full public UI checkpoint suite now passes13/13 (0.50s), formatting applied. No full repository regression/lint yet.

13th slice `plan_adoption`: RED missing plan selector/goal action; GREEN explicit selected producer plan adoption remains0verified, checkpoint preference alone changes no runtime policy, explicit criteria-request draft contains correctly POSIX-quoted actual custom state root and canonical root provider/session even when Parser child selected. Multiple plans fixture uses distinct producers because ImportPlan keeps each producer's latest plan. Cached producer-name registry makes collapsed-child producer labels human-readable. Rapid selection is optimistic/pinned until backend acknowledgment; expansion publishes a completed-agents message so input doesn't race lazy loading.

Parent is delegating runtime/CLI to another implementer. Runtime interface: child module can use private notify, dirty_since, edit/searching; replay_source() returns selected cached (SessionKey,PathBuf) from sources map, not stale backend selection. Main/runtime/executable lifecycle tests are not this agent's remaining files. Runtime will own R interception and one input stream.

Next UI-only work: actual bar/mouse/help/narrowterm/editor presentation, readable delivery history and pending overlay, coalesced control selection/resource cleanup, final API/keymap handoff and scoped/full verification. No real model probes.

## 25 October — UI implementation complete, runtime delegated

UI worktree `/Users/melikhov/Repos/zoetrope-worktrees/zoe-4`, branch `feat/zoe-4-terminal`, clean head `e600ee7047cdae0b3b7803b5877cac4cc67ee105`. Latest integration `3c38d7a` merged as `0d60c8e` before final handoff. No install, push, release, real native mutation/model probe, or source work in main/runtime performed here. Runtime/CLI assigned to sibling `/root/zoe_4_runtime`; root performs combined review/install/PTY/resource checks.

Completed remaining TDD slices:

14. `progress_is_clickable_and_narrow_terminals_keep_help_and_pasted_drafts_usable`: RED missing pointer/help/presentation; GREEN clickable verified Gauge opens criteria, incomplete weighted bar retains an empty cell, narrow stacked layout remains usable, help scrolling, UTF-8 pasted multiline editor/cursor and text shortcut isolation. `0194965`.
15. `pending_deliveries_are_readable_and_acknowledgment_preserves_a_newer_draft`: RED missing readable in-flight history; GREEN `b` delivery pane shows exact human recipient and state before/after delayed peer ACK and across restart. Precise inflight ID overrides only its own Store Sending→Unknown projection. Accepted/Queued guidance clears composer/durable draft only if same target and unchanged text; concurrent newer draft survives. Public additive `WorkUpdate::ClearDraftIfMatches {target,expected_text}` applies under Store's lock. Explicit repeated identical later guidance permitted, no automatic retries and F-key repeats ignored. `0c5f9df`.
16. `background_observation_reaches_later_agents_without_resuming_them`: RED later root in 71 loaded threads never observed before every10s queue reset. GREEN queue retains unfinished IDs across discovery, bounded8/50ms batches yield to explicit control before each read, latest selected metadata coalesced outside32-slot mutation channel, explicit question subscriptions capped256, selected read failure yields Unknown. Actual peer forbids any resume/start. `852ba25`.

Presentation extracted into `src/fleet/presentation.rs`; main FleetConsole still owns cached graph and state, no render/input IO. Final cleanup `e600ee7` groups SendIntent and boxes WorkUpdate in bounded state messages; only expected unused runtime hooks remain until sibling adds `pub mod runtime` and uses them. No warning suppressions in source.

### Final implemented keys

- H/h current/history; / search + Enter; ↑↓ or j/k select; ←→ pan; +/- zoom; o overview; r readable; I internal helpers; Enter/Space expand in graph.
- f edit targeted feedback; F2 explicit guidance, F3 native Queue, F4 Stop. Esc saves and leaves editor; regular shortcut characters remain text. Ctrl-U clear, Ctrl-A/E start/end. Bracketed paste supports multiline UTF-8.
- b saved delivery history, human mode/recipient/state, last32 visible records; PageUp/PageDown detail.
- Q saved questions; L explicitly listen/join; Tab prompt; 1–9 save choice; e freeform answer; F2 explicit current native answer; F3 copy Expired/Resolved answer as feedback; x local dismissal (never native answer/cancellation). Pending/expired/resolved remain honest.
- c criteria; e plain goal title/outcomes; v evidence/explicit attestation; u reopen; F2 commit editor; [ ] choose plan; a explicitly adopt producer plan unverified; p local checkpoint preference; A explicit editable criteria request with correctly quoted actual state root and canonical goal provider/session. Preference alone never pauses runtime.
- d documents; ↑↓ select; Enter explicit exact-path MD Reader request.
- ? scrolling help; Esc back; q navigation quit, Ctrl-C quit. R read-only replay and default CLI/--project are sibling runtime responsibilities.

### Public runtime child seam

`FleetConsole::{new,poll,handle_event,draw,shutdown}` unchanged. Child can access `notify`, `dirty_since`, `edit`, `searching`, `show_help`. `replay_source()` returns selected cached `(SessionKey,PathBuf)` safely. `save_feedback()` clears dirty_since only after successful enqueue; write completion publishes Notify/status, while new edits reset dirty_since. Runtime owns one EventStream and tailer lifecycle, preserves FleetConsole on replay return. Bypass R while edit/search/help active.

### Verification

- All16 fleet UI cases passed together after final cleanup:0.62s.
- Full locked offline cargo test passed at merged `0d60c8e`:274 tests (207lib,9bin,13control,10console,5document,16fleet,14workstate), doc tests0. Final cleanup is a lint refactor;16UI rerun passed afterward.
- `cargo fmt --all --check` passed before final cleanup; cargo fmt applied afterward; git diff --check clean.
- `cargo check --locked --offline --no-default-features --lib` passed.
- Full clippy initially found3 real UI findings, corrected. `cargo clippy --locked --offline --all-targets -- -D warnings -A dead-code` passed, validating all other lints. Unsuppressed check must rerun after sibling runtime consumes notify/sources/replay_source (the3 expected dead-code warnings); no suppressions were added to code. Parent owns final combined release/browser/rustdoc/realterminal/resource checks.
- All invoked sessions finished; no running test/build/peer sessions left. IDs29394/23974/94825/30295/6196 completed. Native question/approval coexistence constraints remain as previously documented, not established by protocol-peer tests.

Potential follow-up review pointers (not blockers added to UI ticket): root's existing backend concurrency audit; runtime-only work without any saved transcript isn't rediscoverable as orphan history after native unload; Store Sending recovery remains a backend audit, with UI exact-local-inflight overlay implemented. Graph/render uses only cached public commentary and supported shell purpose, never hidden reasoning or a model summarizer.
