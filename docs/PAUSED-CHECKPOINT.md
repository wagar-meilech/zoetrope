# Saved checkpoint — 23 October 2026

**Historical checkpoint. Work resumed on 24 October 2026 at the user's request.**
The state below records the earlier pause. The complete customized terminal
interface was not installed at that checkpoint.

## Saved code

Integration branch: `feat/goal-console`, at `b1931b6` before this checkpoint.
Draft PR: https://github.com/wagar-meilech/zoetrope/pull/8. Do not merge into main
or mark ready until the remaining implementation and final review are complete.

| Ticket | Saved result |
| --- | --- |
| ZOE-1 / #2 | Integrated fleet inventory, archived history, stable identities, search and cached graph. |
| ZOE-2 / #3 | Integrated durable goals/evidence, revisions, questions, drafts, outbox, documents and compact local CLI. |
| ZOE-3 / #4 | Integrated local Codex control, exact recipient/turn checks, distinct delivery states and native question lifecycle. |
| ZOE-4 / #5 | Initial terminal slice saved separately as WIP `aca89c4`; not merged. |
| ZOE-5 / #6 | Integrated document launcher and minimal VS Code bridge; installed bridge and real MD Reader preview verified. |
| ZOE-6 / #7 | Usage/architecture documentation started; combined terminal verification, final review and rebuilt installation remain. |

ZOE-4 branch: `feat/zoe-4-terminal`.
Retained clean worktree: `/Users/melikhov/Repos/zoetrope-worktrees/zoe-4`.
Its first public key/render test passes: evidence-backed 1-of-2 progress and `H`
switching history in a real Rataflow graph. This is unfinished scaffolding with
known warnings; do not merge, install or treat it as the completed UI.

Completed implementer worktrees and their separate build caches were removed.
The incomplete UI worktree and its cache are retained for resuming.

## Verified results

- The integrated foundation passes **258 tests**, formatting and portable-core
  checks. Individual final ticket checks also passed warnings-as-errors clippy
  and rustdoc. Existing replay tests remain green.
- The separately locked browser frontend passes its WebAssembly check with
  Rust 1.90. Its existing dependencies require 1.90; the native repo override
  remains Rust 1.88. No dependency pins were changed for that check.
- Real local Codex probes established exact-turn guidance/interruption,
  native queue operations, question replay/resolution and policy-preserving
  joining on disposable sessions. Queueing on idle work can immediately start
  it. Second-client answering, competing answers and real approval arbitration
  remain unverified; native approvals stay in Codex.
- The installed runtime reports direct input available for the coordinator and
  unavailable for observed spawned helpers. Guidance/queue respect that;
  interruption is separately addressed to an exact thread and turn. No hidden
  child-to-parent fallback is allowed.
- A real VS Code MD Reader webview rendered the selected goal-console spec
  through the installed `wagar.zoetrope-preview` bridge. Preview dispatch status
  still makes no general render-acknowledgment claim.
- A synthetic debug-build public-interface resource probe held about 64 MiB
  of local history across 128 records with **21.2 MB peak RSS**. Unchanged
  refreshes averaged **2.77 ms**. These are fixture observations; the complete
  terminal application's idle CPU/memory have not yet been measured.

`~/.local/bin/zoe` still points to the original source-built release binary in
the main checkout. It has **not** been rebuilt with these customizations. The
new user guide describes the intended workflow, including UI work still pending.

## Resume from here

1. Continue ZOE-4 in its retained worktree. Merge the latest integration tip
   before further work, including documentation commit `b1931b6`. Continue
   vertical TDD at the already-approved public console and executable seams;
   no new approval/interview is needed.
2. Finish selection/expansion/search, human-readable public activity, persistent
   steering/questions, editable criteria/evidence, document actions, live
   status overlay, default fleet launch and `--project` compatibility. Keep one
   input owner for replay and return. Use independent bounded workers for
   inventory/state, control and document opening; event-driven idle rendering.
3. Use the merger agent to integrate the completed ticket. Preserve both
   `state` and `control` CLI dispatches. Keep the incomplete WIP off integration.
4. Complete ZOE-6: update final keyboard documentation, run final repository
   checks and package dry run, review the entire combined change, fix review
   findings via one implementer, build/install the release, and verify real
   terminal/replay/document flows plus idle resource use.
5. Push the final integration, mark PR ready, and remove completed worktrees.
   Do not merge the PR into main without a user request.

The approved requirements and answers remain in
[the spec](GOAL-CONSOLE-SPEC.md), [task graph](GOAL-CONSOLE-TICKETS.md),
[review approvals](IMPLEMENTATION-REVIEW.md) and
[answered questionnaire](CUSTOMIZATION-QUESTIONS.md). The user explicitly wants
fast, resource-light code: Rust for the app; only a tiny JavaScript editor
bridge; bounded caches/reads; no hidden summarizing model or extra GUI service.

Detailed local notes are in `/Users/melikhov/Documents/Research/`:

- `zoe-4-implementation.md`: exact WIP state, last green test and next slices.
- `zoe-4-ui-handoff.md`: UI, worker, replay-lifecycle and public-activity guidance.
- `zoe-1-implementation.md`, `zoe-2-implementation.md`,
  `zoe-3-implementation.md`, `zoe-5-implementation.md`: final APIs and TDD evidence.
- `zoe-control-compatibility-2026-10-23/FINDINGS.md`: installed protocol evidence,
  exact schemas, transcripts and compatibility limits.
- `zoetrope-merge-notes.md` and
  `zoetrope-integration-validation-2026-10-23.md`: integrated checks and real preview.
- `zoetrope-build-status.json`: machine-readable paused state.

The invoked workflow skills are `grilling`, `tdd`, `to-spec` and `implement-spec`;
their approved process includes separate implementer worktrees, a merger,
GitHub issues/draft PR, final code review and no automatic merge into main.
