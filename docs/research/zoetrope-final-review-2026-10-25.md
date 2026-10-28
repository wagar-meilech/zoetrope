# Final goal-console code review — 25 October 2026

> **Resolution update:** All R1–R8 findings below are cleared after independent re-review of merged integration `47b41e1` (source fix `a1ed060`). Follow-up delayed-copy, orphan-edge and held-answer-save cases are also fixed. Native290 tests and portable176 tests pass. The original findings/evidence are retained as review history; see the resolution record at the end.

Reviewed integration `d853af2` against `origin/main`, following the explicitly invoked engineering code-review skill at `/Users/melikhov/.codex/plugins/cache/claude-cowork/engineering/1.1.0/skills/code-review/SKILL.md`, CLAUDE, the approved goal-console spec and answered questionnaire. This is an independent review after merging ZOE4, not an implementation handoff. No repository source changes, live controls, model calls, settings changes or native-history writes were performed.

**Recommendation: fix the findings below before the installed release/ready-for-review handoff.** The existing278-test suite is green, but it does not cover these external-update, sensitive-input, backpressure and actual-format cases.

## Dimension ratings

| Dimension | Rating | Assessment |
| --- | --- | --- |
| Security | Needs changes | Native secret-answer metadata is honored in the saved question summary but lost while editing/converting the answer. No shell interpolation or new Internet transport found. |
| Correctness | Needs changes | External goal edits do not refresh selected state; save backpressure can discard drafts; overflow silently loses steering text; a cache/stamp race can retain stale state. |
| Performance | Mostly sound, one scaling fix | IO workers, bounded channels/read budgets, cached graph and idle rendering are appropriate. First placement still scans all occupied positions per new node on the UI thread. |
| Maintainability | Acceptable with targeted cleanup | Provider/control/store seams remain distinct and public-seam tests are useful. The large fleet coordinator needs explicit save acknowledgments and record generations rather than more implicit cache assumptions. |

## Findings, in priority order

### R1 — P1: selected progress and work never refresh after external agent/CLI updates

**References:** `src/fleet/mod.rs:348`–353 (change comparison omits summaries), `:372`–381 (selected full record only loaded for selection change/missing state), `:393`–423 (selected state replaced only by StateWorker output), `src/fleet/presentation.rs:106`–139 (visible progress derives from that stale selected record).

The inventory refresh reads new local goal state, but polling does not notice a summary-only change and never reloads the already-selected WorkRecord. An agent's `zoe state ... verify` can therefore leave the visible selected bar/count at0/2 indefinitely. Even an incidental later graph sync can show a newer card beside a stale detail bar. The same missing selected-record invalidation affects externally edited criteria, questions and documents.

**Public-interface proof:** with one real-format fixture goal selected at0/2, applied Verify through the public Store seam, polled for7seconds (longer than the5second refresh cadence), then rendered the actual FleetConsole: `old_zero_visible=true`, `new_one_visible=false`. No internal fields were inspected.

**Fix direction:** carry a compact, trustworthy per-record generation/stamp in inventory updates; compare summaries as display state and reload the selected record off-thread when its version changes. Avoid putting every full historical record back into the inventory view. Cover external progress and an external selected-record update that leaves its compact summary unchanged.

### R2 — P1: secret answers are rendered in clear text while editing and after copy-to-feedback

**References:** `src/fleet/mod.rs:1067`–1086 (Answer editor drops prompt sensitivity), `src/fleet/presentation.rs:229`–233 (wraps/displays raw editor text), `src/workstate.rs:870`–900 (converts every answer into ordinary feedback text), `src/fleet/mod.rs:398`–406 (opens that unmarked feedback editor). Saved question masking at `mod.rs:1191`–1198 is correct but insufficient.

A prompt with `is_secret=true` displays “Saved (hidden)” in the question pane, but pressing e immediately displays the saved value. An expired/resolved secret answer copied into feedback is also exposed because sensitivity/provenance is lost. Direct Answer delivery text is already omitted from the delivery pane; preserve that protection through all paths.

**Public-interface proof:** an isolated secret-marked prompt containing a sentinel was rendered through normal Q/e input, then copied through F3: both `SECRET_EDITOR_EXPOSED=true` and `SECRET_COPY_EXPOSED=true`.

**Fix direction:** carry secret/sensitive state into the editor and mask the displayed value/cursor consistently; retain sensitivity when converting/saving/sending feedback, or explicitly refuse unsafe conversion of secret fields. Ensure saved deliveries and subsequent reopen also cannot expose the value. This finding does not require encrypted local storage; existing0600 files are appropriate to the agreed local persistence scope.

### R3 — P2: oversized paste silently drops the end of a steering instruction

**References:** `src/fleet/editor.rs:12`–29 (`insert` stops at65,536bytes and reports only success), `src/fleet/mod.rs:1580`–1590 (reports ordinary saving afterward). Typed-character limits at `editor.rs:119`–128 also provide no limit feedback.

A long paste is partially accepted and auto-saved as though intact. A constraint near the end can vanish before guidance is sent. A bounded editor is desirable, but silent partial mutation is unsafe for a steering surface.

**Public-interface proof:** pasted65,536 X bytes followed by `DO_NOT_TOUCH_DATABASE`. Persisted draft length was65,536; the trailing constraint was absent, and no limit/truncation message was visible.

**Fix direction:** reject an over-limit insertion atomically and keep the previous draft with a clear human-readable limit message, or explicitly mark partial input and prevent sending until the user reviews it. Test UTF-8 byte boundaries and preservation of the original text.

### R4 — P2: Esc/F2 can discard an answer or feedback draft when saving fails or the queue is full

**References:** `src/fleet/mod.rs:721`–735 and `:743`–759 (enqueue failure leaves dirty data only in editor), `:828`–830 (Answer commit returns true regardless of save result), `:903`–912 (closes editor), `:1662`–1666 (Esc unconditionally discards editor), `src/fleet/state_worker.rs:49`–59 (actual disk failure arrives only after the editor was closed).

The queue-busy message says the draft remains in the composer, but Esc immediately removes that composer. Answer F2 has the same issue. A successfully queued write can also fail later; there is no retained pending edit/acknowledgment to restore. Once navigation/shutdown occurs, the only copy is gone. With dirty_since still set and edit removed, a rejected enqueue can additionally leave a useless autosave wake loop until another edit.

**Public-interface proof:** held the actual local state-file advisory lock, filled the bounded worker queue using ordinary f/Paste/Esc events, released the lock, let it drain, and shut down. Result: `SAVE_BUSY_REPORTED=true`, `LAST_DRAFT_STILL_VISIBLE=false`, `LAST_DRAFT_PERSISTED=false`. Isolated disposable files only.

**Fix direction:** make save completion explicit; retain the exact target/question/prompt and text until a matching durable success acknowledgment. Do not close/drop on failed enqueue. On asynchronous write failure keep/reopen the edit, and ensure shutdown flushes pending data rather than silently discarding it. Cover both queue pressure and an actual write error through the public UI/state seam.

### R5 — P2: old work can be cached under a newer concurrent writer's stamp

**References:** `src/console.rs:204`–206 (`load_work` read then path stamp), `:304`–306 (`update_work` after apply releases the lock), `:360`–363 (recorded-work import), `:326`–328 (later refresh skips matching stamp). `Store::stamp` is a separate path metadata call at `src/workstate.rs:490`–493.

Interleaving: read/apply returns recordA; another process atomically writes B; the subsequent path stamp belongs to B; cache_work stores A with B's stamp. Every unchanged refresh then skips the read, leaving the local summary/search/progress stale until another disk write. Large record reads widen the read/stamp race. The pre-read stamp ordering in refresh_work is safer because a race causes another refresh, not permanent stale caching.

**Fix direction:** return a stamp/generation coupled to the record actually read/written (for reads, use the opened file handle's metadata); or explicitly invalidate its version while maintaining accurate byte accounting. Keep concurrent atomic writers and the cache byte/entry bounds. This finding is established by the final code's lock/read/stat ordering; no private-method test or artificial call-count assertion is needed.

### R6 — P2: the installed Codex format's public commentary is ignored

**References:** `src/provider/activity.rs:27`–30 (requires payload.channel commentary/final), `:39`–40 (only then records readable public narration).

The real0.164 rollout inspected by root and independently checked by this review uses `payload.phase="commentary"` / `"final_answer"` on assistant response_item/message records, without a channel field. All75 explicitly public messages in the inspected own rollout are ignored by the new activity projection, leaving generic tool labels/long command text as the principal activity instead of the desired human narration.

**Read-only schema evidence:** counted only field names/type metadata, without printing any transcript content:64 records `(channel:null,phase:commentary)` and11 `(channel:null,phase:final_answer)`. This is the already-known own task rollout, not another user's session.

**Fix direction:** support both documented/observed explicit public channel and phase forms, including final_answer. Continue to exclude analysis/thinking/reasoning and unknown classification; never use the legacy conflated Reasoning fact. Add real-format public-seam fixtures for both forms.

### R7 — P2: a held copy key can turn a local copy into native Queue submission

**References:** `src/fleet/mod.rs:1763` (F3 copies a closed question), `:398`–406 (copy completion automatically switches into Feedback), `:1629`–1631 (the same F3 now queues), `:1601`–1603 (only explicit KeyEventKind::Repeat suppressed).

On terminals without keyboard-enhancement repeat metadata, autorepeat arrives as additional ordinary Press events. After the first F3 copy finishes, the next repeat has changed meaning and can submit the copied answer to the runtime's native queue. Queue may start idle work immediately. The current repeat-kind check therefore does not ensure the promised local-only conversion.

**Fix direction:** use a distinct copy action key from the native send/queue keys, or keep copy completion in a non-sending pane until another deliberate navigation action. Do not rely on Repeat metadata that conventional Unix terminals do not supply. Cover consecutive Press-form copy input after the asynchronous save finishes using a peer that rejects any native mutation.

### R8 — P2 performance: initial graph placement scans every previous position for every node on the UI thread

**References:** `src/fleet/mod.rs:1459`–1467 (`self.positions.values().any` per new visible node), invoked synchronously from poll's sync_graph.

Opening a large history makes this O(n²) even when all proposed coordinates are unique: the no-collision case still walks all previous positions. Metadata indexing is bounded/off-thread, but this avoidable work still occurs in the terminal lane just when the user opens the global history. Node count is intentionally not limited by the approved history view.

**Fix direction:** keep a set/index of occupied coordinate slots for constant/logarithmic collision checks while retaining the positions map for stable identities and dragged positions. A representative public fleet/render benchmark is preferable to a timing-fragile CI assertion. This is a source-level scaling finding; no claim of measured production latency is made.

## Positive observations and candidates cleared

- Exact native thread and expected-turn validation is explicit; no child-to-parent redirect, replacement daemon, hidden model summarizer, permission-policy override or automatic unknown-delivery retry was found.
- Native queue semantics are honestly described; idle submission may start immediately. Runtime acceptance and verified goal completion stay separate.
- Weighted incomplete progress can no longer round to100%; Gauge and card bars reserve an unfinished cell. Prior candidate cleared.
- Criteria request drafts include the correctly quoted actual state root and canonical provider/root-session even when an agent is selected. Prior candidate cleared.
- One owned EventStream serves fleet/replay, with bounded retiring replay loading and no idle60fps redraw timer. Real PTY lifecycle tests are meaningful.
- Public activity is separate from protected reasoning, with a dedicated typed provider boundary. The missing actual phase shape is a compatibility gap, not evidence of reasoning leakage.
- Transcript/live question identity is reconciled with itemId/call_id; source output resolutions are imported without manufacturing native pending IDs. Expired/resolved drafts can remain available without claiming original delivery.
- Atomic concurrent local updates, OS advisory locks,0600 record/temp files and a bounded full-record cache are sound foundations. Existing cache-stamp and read-projection caveats should not be confused with lost-update protection in the locked writer itself.
- Current UI correctly overlays its exact in-flight delivery ID, and receipts preserve a newer draft when an older submission completes. The prior general Sending→Unknown concern is mitigated for this active UI; no additional blocker is raised for conservative CLI recovery semantics.
- The Rust document launcher passes arguments directly, validates regular Markdown files, preserves exact path spelling, and reports dispatch rather than claimed rendering. The small extension validates local URI form and serializes preview requests. No bridge dependency/service expansion found.
- Some goal title duplication between session/coordinator cards is a presentation tradeoff, not a correctness blocker.

## Small pre-existing test-gate cleanup

`src/state/mod.rs:1230` references native-only crate::handler from an otherwise portable test. Optional `cargo test --no-default-features --lib` consequently fails to compile, although the required portable library build passes. This is unchanged upstream code, not introduced by this PR. Adding the appropriate native cfg to that test is a useful small final cleanup; do not remove portable production behavior to hide the error.

## Verification evidence

- Combined branch already passed278 tests, strict clippy, formatting, native docs and portable-library checks before this review; merger independently repeated full278 tests/fmt/core at integration.
- Targeted review probe: `docs/research/probes/zoetrope-review-public-probe.rs` / executable of same stem. Output:

```text
AFTER_EXTERNAL_UPDATE old_zero_visible=true new_one_visible=false
SECRET_EDITOR_EXPOSED=true
SECRET_COPY_EXPOSED=true
PASTE_SAVED_BYTES=65536 TAIL_CONSTRAINT_RETAINED=false LIMIT_MESSAGE_VISIBLE=false
```

- Backpressure proof: `docs/research/probes/zoetrope-review-save-queue.rs` / executable of same stem. Output:

```text
SAVE_BUSY_REPORTED=true LAST_DRAFT_STILL_VISIBLE=false
LAST_DRAFT_PERSISTED=false
```

Both probes used only public FleetConsole key/render/poll and Store interfaces with disposable fixture homes; they cleaned their data. The second uses an actual held OS advisory file lock to model a concurrent/stalled writer, not an internal mock. No broad test rerun was needed to establish these gaps. Repository working tree remains unchanged.


## Independent re-review and resolution — integration47b41e1

Merged final clean fixer branch `8e58b8b` (source `a1ed060`, containing documentation integration `b454397`) with a Conventional non-squash merge as `47b41e1`. No merge conflicts, source tweaks, push, installation or history rewrite performed by merger. Read the final committed source against each original finding and exercised the merged tree through the existing full suite plus independent public probes.

**No outstanding blocker remains from this review.** Security/correctness now pass the reviewed scenarios; performance uses indexed placement and batched geometry; maintainability is improved by explicit save acknowledgments and compact selected-record version invalidation. Real installed-terminal/runtime/editor QA and final release/PR handling remain root responsibilities.

| Finding | Final evidence and disposition |
| --- | --- |
| R1 external selected work | Cleared. `fleet/mod.rs:367` compares compact selected versions, summaries participate in redraw, and a bounded retained reload intent reaches StateWorker even under queue pressure. Public test changes external progress, then an external document whose compact summary is unchanged. Independent probe now reports old_zero_visible=false/new_one_visible=true. |
| R2 secret answers | Cleared. `fleet/editor.rs:43`–44 masks secret display; `fleet/mod.rs:1217` carries prompt sensitivity into the editor; `workstate.rs:870`–878 rejects nonempty secret-answer conversion before any outbox write. Public regression covers edit/paste/save/reopen/restart and conversion refusal. Independent sentinel no longer appears; copy is explicitly refused. f in Questions uses the exact question producer. |
| R3 silent overflow | Cleared. `fleet/editor.rs:17` preflights sanitized UTF-8 insertion and returns an error before changing text/cursor; key and paste paths show the64KiB limit. Public regression and independent probe preserve the complete previous draft after an oversized paste. |
| R4 failed-save data loss | Cleared. `fleet/mod.rs:834` has one identified in-flight editor save; `:471` consumes its dedicated durable result; requested close waits for the matching success, and changed text is saved again. `state_worker.rs:64` retains this acknowledgment independently of loads/other writes. Disk/queue failures keep the composer and retry message. Answer, feedback and criteria editors are covered. `try_shutdown` at `mod.rs:1995` uses an ordered explicit flush and returns an error without stopping workers on failure; runtime Ctrl-C goes through the same acknowledged close path and observes asynchronous quit. Full suite covers real OS-lock pressure and actual failed filesystem writes followed by repair/retry. Independent lock replay retains the final sentinel through orderly shutdown. |
| R5 cache/stamp race | Cleared by source ordering. `console.rs:210`,`:327`,`:349`,`:383` capture versions before reads/writes. A concurrent replacement can now cause another reload but cannot pin an older record to a newer stamp. `cache_work:224`–239 counts the actual encoded record without allocating another copy, so the safer pre-read stamp does not undermine memory accounting. |
| R6 actual public narration | Cleared. `provider/activity.rs:30`–33 supports explicit public phase forms when channel is absent; a present channel remains authoritative. Public real-format fixtures prove commentary/final_answer visible while analysis/unknown remain absent. |
| R7 copy/send repeat | Cleared. F5 is local copy; F3 remains Queue. Public protocol-peer test sends repeated ordinary Press-form F5 after asynchronous conversion and permits no native mutations. Adjacent F2 Answer save/submit boundary also fixed: `mod.rs:971`–975 saves in place, leaving Esc as deliberate return; a peer test proves held Save cannot become native Submit. |
| R8 placement scaling | Cleared. `mod.rs:1589`–1597 indexes visible IDs/occupied slots once; `:1713`–1714 uses rataflow's existing whole-batch node/edge APIs, avoiding repeated hierarchy resolution. Node positions remain keyed to stable identities, and hidden/dragged positions remain reserved. Final manual benchmark has no CI timing assertion. |

### Follow-up cases discovered while re-reviewing fixes

1. **Delayed copy overwrote a newer composer:** independently reproduced against the initial fix worktree with an actual held state lock. The single fixer added a public regression and the final code at `mod.rs:424`–440 now preserves any existing composer/target/dirty text while retaining the older copied answer durably in the outbox. Repeating a copy while pending is coalesced. Independent rerun against the merged native library reports the newer draft visible and persisted.
2. **An orphan parent invalidated the entire batch edge set:** rataflow validates all endpoints before replacing edges. The final code filters to visible non-self parent endpoints (`mod.rs:1698`–1701) and coalesces duplicate node identities. A public orphan-plus-connected-goal regression proves ordinary edges remain present without fabricating parentage.
3. **Held F2 save could become native answer submission:** final Answer editor save remains in-place until a separate Esc; the full native-peer regression passes.
4. **Portable test cfg:** the unchanged upstream native-handler-only test is now correctly native-gated. Required portable build and176 portable library tests pass; no portable production behavior removed.

### Merged checks independently run

- `cargo fmt --all --check`: pass.
- `cargo check --lib --locked --no-default-features`: pass.
- `cargo test --locked --all-features`:290 passed,0failed,1ignored manual large-history resource probe. Breakdown:207library+8binary+13control+10console+5launcher+5actualPTY+28fleetUI+14workstate. FleetUI regressions passed together10.71s; actual PTY cases1.15s. Scoped escalation allowed disposable local Unix-socket peers; no installed runtime mutation.
- `cargo test --locked --no-default-features --lib`:176 passed,0failed.
- Fixer independently passed all-target/all-feature strict clippy at the identical source tip; root owns the final release/docs/wasm/packaging repetition.

Independent public probe output against the merged native library:

```text
AFTER_EXTERNAL_UPDATE old_zero_visible=false new_one_visible=true
SECRET_EDITOR_EXPOSED=false
SECRET_COPY_REFUSED=true
LIMIT_MESSAGE_VISIBLE=true
OVERFLOW_PRESERVED_PREVIOUS_DRAFT=true

BEFORE_COPY_ACK_NEW_VISIBLE=true
AFTER_COPY_ACK_NEW_VISIBLE=true OLD_COPY_VISIBLE=false
AFTER_COPY_ACK_NEW_PERSISTED=true

LAST_DRAFT_PERSISTED=true
```

Probe sources: `docs/research/probes/zoetrope-review-fixed-public-probe.rs`, `docs/research/probes/zoetrope-review-copy-race.rs`, `docs/research/probes/zoetrope-review-save-queue-fixed.rs`. The last probe waits for selected details before input; the corrected implementation serializes acknowledged saves rather than allowing the former queue-fill pattern, and its final exact sentinel survives shutdown. Terminal hard-wrap may split a sentinel on screen, so durable text is the authoritative assertion there. All fixture homes cleaned; source tree remained clean throughout.
