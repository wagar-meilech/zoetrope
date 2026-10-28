# Zoetrope final review fixes — 25 October 2026

Worktree `review-fixes`, branch `feat/zoe-review-fixes`, base `d853af2`. Single post-review implementer. Approved public console and executable seams from docs/IMPLEMENTATION-REVIEW.md; following TDD skill plus tests/mocking guidance. No native runs or history writes.

## R1 and R5 — external work invalidation and stamp ordering

RED: `cargo test --locked --test fleet_ui selected_goal_refreshes_external_progress_and_details_without_navigation` failed waiting for externally verified progress. GREEN: same test passes, including a later external document addition that does not change the compact summary. Inventory carries only compact record versions; selected details reload on the state worker. Summaries participate in redraw comparison. Record cache stamps are captured before read/apply/import; a concurrent replacement can trigger another refresh but cannot tag an older cached record with the newer replacement's stamp. Cache byte/entry accounting remains intact. R5 established by explicit source ordering; no implementation-coupled or timing-fragile test added.

## R2 — secret answers

RED: `cargo test --locked --test fleet_ui secret_answers_stay_hidden` exposed PRIVATE_SENTINEL through normal Q/e. GREEN passes editing, pasted input, save/reopen, restart and rejected feedback conversion. Secret editor display/cursor use masked characters. Both UI and public Store explicitly refuse copying nonempty secret answers into ordinary feedback; UI explains the safe alternative (f to write new non-secret feedback). No outbox/draft is created by refusal. Direct native answer delivery remains omitted from delivery text. Local storage retains existing private-file contract.

## R3 — bounded paste

RED: `cargo test --locked --test fleet_ui oversized_paste` showed silently truncated UTF-8 input without limit notice. Atomic insertion now preflights sanitized UTF-8 byte size, preserving previous text and cursor on overflow. Typed character and Enter input share this path; human message states 64 KiB limit.

R3 GREEN: the UTF-8 overflow test passes and preserves exactly `Keep this constraint` after save/restart through Store. Full input remains bounded to65,536 sanitized UTF-8 bytes; rejected insertions make no text/cursor mutation.

## R4 — durable save acknowledgment and backpressure

RED: `cargo test --locked --test fleet_ui feedback_stays_in_the_composer` lost the visible composer when the queue was full. GREEN retains it and ordered shutdown flushes its exact text. The analogous answer pressure test verifies exact question/prompt persistence too. Pressure uses an independent Python process holding the real OS advisory lock; the initial same-process lock fixture was replaced because parallel process activity made its pressure precondition unreliable.

Actual-write-error coverage temporarily replaces the real state directory with a regular file. Feedback and answer editors remain visible after failed asynchronous save; `try_shutdown` returns an error and retains text; repair + Esc persists the exact draft. The worker has one dedicated editor acknowledgment slot, not an unbounded completion log; only one editor save is in flight, while the existing32-command queue remains bounded. Input changed during a save is saved again before a requested close. Enqueue success never claims durable success. Failure stops the autosave deadline until edit/retry; runtime does not spin on a pending acknowledgment. Ctrl-C in the actual fleet runtime now uses that same save-before-quit path. Direct shutdown uses an ordered blocking flush outside the interactive lane and reports failure.

Additional RED→GREEN: `criteria_edits_also_remain_available_when_storage_rejects_a_save` exposed the same early close for goal/evidence edits; they now use the same acknowledgment path. Criteria Esc still deliberately cancels an uncommitted edit. If a criteria save is pending/failed, shutdown reports that it is not confirmed rather than discarding it.

The existing delivery test now waits for “Draft saved” after Esc before navigating, matching the visible durable-close contract. The actual terminal paste test exposed an early-f race while selected details were still loading; feedback-open intent is retained until those details arrive (and canceled by a deliberate new selection), instead of silently ignoring the key.

Merger follow-up RED→GREEN: `a_delayed_answer_copy_preserves_a_newer_feedback_composer_and_its_target`. A real external state lock delays F5 conversion; f + new text before copy acknowledgment previously disappeared. Copy completion now preserves any existing composer, its target and dirty text, while keeping the older copied answer in its durable local outbox record. A repeated copy before completion is coalesced.

## R6 — actual Codex public phase format

RED: `cargo test --locked --test fleet_ui explicit_codex_public_phases` failed to show the explicit public result. GREEN accepts phase `commentary` / `final_answer` when channel is absent, while preserving explicit public channel support. A present channel takes precedence: analysis cannot be reclassified by a phase. Synthetic real-format fixtures check public commentary/final_answer alongside analysis and unknown records; protected/unknown content stays absent. Existing channel+shell-purpose test remains green.

## R7 — copy/send key separation

RED: `cargo test --locked --test fleet_ui repeating_the_local_copy` could not copy via the new separate local key. GREEN: F5 copies an expired/resolved question into feedback locally; F3 remains Queue. The public test uses an independent disposable Codex protocol peer that rejects every mutation, then sends12 ordinary Press-form F5 events after async completion. No native submission occurs. Existing explicit-copy tests and help hints are updated to F5. Secret copy refusal remains available via F5; f from a question targets that question's exact agent for writing new non-secret feedback.

## R8 — indexed placement and batched geometry

Occupied coordinates use a hash index with normalized zero coordinates; identities/dragged/hidden positions remain in the existing position map. The public manual history probe (`large_fleet_placement_resource_probe`, ignored by ordinary CI with no timing assertion) first found a second quadratic cost in this same path: rataflow's single-node add/position methods resolve every prior node's geometry each time. The implementation uses its existing `set_nodes`/`set_edges` batch APIs instead.

Representative debug-build observations on this Mac (1,000/4,000 archived goals, one project):

| Stage |1,000 peak UI poll |4,000 peak UI poll |
|---|---:|---:|
| Occupied index alone, per-node geometry |1,629.4ms |28,137.4ms |
| Occupied index + batched geometry |33.5ms |144.3ms |

End-to-end public startup after batching was359ms/1,072ms, including metadata discovery. The probe verifies that an unchanged refresh produces the same rendered graph. These are representative debug measurements, not CI thresholds or production latency guarantees.

Merger follow-up RED→GREEN: `an_orphan_agent_does_not_remove_the_connections_for_other_visible_work` rendered only one of the expected two arrows because a missing parent caused batch edge validation to reject the entire replacement. Batch edges now include only visible, non-self endpoints; orphan nodes retain their real metadata/visibility. Duplicate node identities are coalesced before batch construction, matching prior add-node behavior.

## Portable cleanup

Native cfg is added only to unchanged upstream `user_pan_cancels_glide`, whose test calls the native handler. `cargo test --locked --no-default-features --lib` now compiles and passes176 portable tests; no portable production behavior is removed.

## Final controls and behavior

F2 guidance / answer submission or editor save as labeled; F3 native Queue; F4 exact-agent Stop; F5 local closed-question copy. Esc closes persistent feedback/answer editors only after durable success. A storage failure retains the editor with retry text. Criteria Esc cancels a deliberate uncommitted edit. Secret values are masked in their answer editor and saved question pane, and ordinary feedback conversion is refused for nonempty secret fields. No model calls, services, dependencies, Internet requests, native-history writes, policy changes, installer/release build, push or history rewriting were performed by this implementer.

Further R7 safety RED→GREEN: `saving_an_answer_keeps_the_editor_open_so_held_save_cannot_become_submission` verifies a pending native question with the independent peer. F2 saves an answer and keeps its editor open, matching its existing hint. Ordinary repeated F2 Press events remain Save. Only Esc closes the editor after durable storage; a later deliberate F2 from the question pane submits. This avoids the same save→submit meaning switch while preserving explicit native submission.

Final source commit: `a1ed060 fix(ui): preserve steering edits and refresh goal progress`. Merged latest integration documentation (`b454397`) into this branch using a Conventional merge commit. No installer, release build, push or history rewriting.

## Final verification and handoff

Clean branch tip `8e58b8b chore(ui): merge final goal console guidance` includes `a1ed060` source and latest integration documentation `b454397`.

- `cargo fmt --all --check`: pass.
- `cargo clippy --locked --all-features --all-targets -- -D warnings`: pass.
- `cargo test --locked --all-features --quiet`:290 passed (207 library,8 binary,13 protocol,10 console,5 launcher,5 actual terminal,28 public fleet UI,14 state); one manual resource probe ignored in ordinary run.
- Final committed public resource probe:1,000 goals peak UI poll25.2ms/startup428ms;4,000 peak UI poll108.5ms/startup1,263ms. Stable unchanged render assertion passed. No timing threshold is imposed in CI.
- `cargo test --locked --no-default-features --lib --quiet`:176 portable tests passed.

All commands used isolated `CARGO_TARGET_DIR=/Users/melikhov/Repos/zoetrope/target/review-fixes`. Native protocol/terminal tests requested only disposable local IPC access. Root remains responsible for merger re-review, release installation, final real-terminal/editor/manual/resource checks and PR/history housekeeping. No local source changes remain.
