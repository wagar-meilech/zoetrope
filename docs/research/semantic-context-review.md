# Semantic context two-axis review

Initial review: `ec35ad3`; final affected-source follow-up: `856c565`. Read-only independent reviews; runtime/check evidence is recorded separately in the validation document.

## Standards

### Original findings

Reviewed `git diff a5b9d5eb274c9a53012062348dc6e4c50ad4cb2a...HEAD` at `ec35ad3649b4b9927f3a2dbb776f9644992ef571` (12 commits). Read-only review; no tests or native actions run.

1. **Documented architectural breach — provider status spelling reaches the UI.** `src/fleet/work_view.rs:56–61` adds `match plan.steps[at].status.as_str()` with `"in_progress" | "inProgress" => "in progress"`. These provider spellings arrive unchanged through `serde_json::from_value::<Vec<PlanStep>>` in `src/provider/work.rs:80–82`. `CLAUDE.md:20` requires that “nothing past” `src/provider/` knows the transcript format; `docs/ARCHITECTURE.md:29–45` requires providers to translate records into shared vocabulary. Normalize native spellings at ingestion into a shared status representation; keep UI labels in the presentation layer. The raw string field predates this diff, but the new UI interpretation is the changed hunk crossing the boundary.

2. **Judgement call — possible Primitive Obsession / Duplicated Code.** `src/fleet/work_view.rs:157`, `:201–208` and `:278–285` choose work using rendered text: `entry.status() == "in progress"` and `entry.status() == "blocked"`. The compact/full current-entry cascade is copied, while `focus_work` independently chooses active/first. Editing display wording can silently change selection priority, and duplicate policies can drift. Expose semantic predicates or a small normalized status enum and centralize current-entry selection, with display labels separate. This is a baseline smell heuristic, not a documented hard rule.

No other documented-standard violations found in changed implementation, tests or probe sources. New tests use the approved public seams; state APIs prepare fixtures, and executable reads verify persistence. Vertical red/green evidence is recorded in SEM-1/2/3 notes. Tooling-enforced issues were excluded as instructed. Final combined checks/resource evidence remain the coordinator's responsibility.

**Count: 1 documented breach; 1 maintainability judgement.**

### Final resolution and verdict

Focused read-only follow-up reviewed `git diff ec35ad3...HEAD` at `a63850abe1457ba26bf625007604f9903bcd7d95`, the final documentation, and `docs/research/semantic-context-review-fixes.md`.

1. **Provider-boundary finding closed.** `src/provider/work.rs:14–20` owns native/legacy spelling interpretation. The shared `PlanStatus` vocabulary separates known semantics from `Unknown(String)`; its legacy decoder delegates to that provider function (`src/workstate.rs:188–233`). UI logic now matches enum variants (`src/fleet/work_view.rs:47–55`). The saved/context string codec retains the existing JSON vocabulary and preserves unknown/absent statuses without granting completion semantics. Codec spelling is local persistence compatibility, not UI knowledge of transcript aliases.

2. **Maintainability finding closed.** `current_entry` (`src/fleet/work_view.rs:88–101`) centralizes explicit-selection/active/blocked/first precedence. Work focus, selected-owner lookup, compact inspection and full inspection use it. Selection compares semantic `WorkStatus` variants; `status_label` owns display wording. Expired explicit selections remain unavailable until deliberate work selection.

The additional empty-plan and relative-state-root fixes respect the documented boundaries and approved public testing seams. Reviewed machine-applied condition/match simplifications introduce no new standards issue. Public regressions cover withdrawal, cross-directory command identity, the legacy alias, unknown status and absent status. Recorded evidence reports 317 native tests and strict Clippy on Rust 1.99/1.88; this follow-up did not rerun them or perform native actions.

Final narrow follow-up reviewed `git diff a63850a...HEAD` at `856c5654020a2829f4d3fb3214a7fa1660b8225a`. Native startup captures `current_exe` before workers (`src/main.rs:338–343`); shared command construction validates absolute/UTF-8 paths and shell-quotes executable/root (`src/fleet/mod.rs:1652–1678`). Both T/A drafts reuse it; A now requests compact context (`:1763–1765`). No drawing-time IO, new dependency, implicit send or new hard standards issue was introduced. Public PTY and context regressions exercise fake-PATH/cross-directory execution and private-material omission. The coordinator reports the exact committed tree passes 319 native tests and strict Clippy on both toolchains; this review did not rerun them.

**Final Standards verdict at `856c565`: clear. Zero remaining documented breaches or maintainability findings from this review.**


## Spec

### Original findings

Reviewed `git diff a5b9d5eb274c9a53012062348dc6e4c50ad4cb2a...HEAD` at `ec35ad3649b4b9927f3a2dbb776f9644992ef571`. Read the approved specification, mechanism, questionnaire, task graph and `CLAUDE.md`. Final reporting documentation is in flight and excluded from missing-code findings.

(a) Missing/partial requirements: no additional confirmed gaps beyond the incorrect behaviors below.

(b) Unasked behavior/scope creep: none found.

(c) Implemented behavior that appears wrong:

- **P2 — Empty native plans preserve withdrawn work.** Spec `docs/SEMANTIC-CONTEXT-SPEC.md:45`: “Native plan changes are revisions; title/index similarity does not prove durable identity.” `src/provider/work.rs:86–88` discards `update_plan` when `plan` is empty. A newer empty plan therefore leaves the previous proposal ID, active steps and source time persisted. The new work view continues showing that previous work and a selected step never expires. A standalone public `Console` probe (`/tmp/zoetrope-semantic-empty-plan.rs`) appended `{plan:[]}` after an active plan and confirmed refresh retained the old proposal unchanged. Import the empty revision so its earlier selection expires.

- **P2 — Tracking commands can address a different state store.** Spec `docs/SEMANTIC-CONTEXT-SPEC.md:50`: “Enable tracking prepares an editable coordinator instruction with exact provider/session/state location and transition-only reporting commands.” `src/fleet/mod.rs:1656–1671` copies `state_root` verbatim into the instruction. Relative roots are supported by `Store::default_root` (`src/workstate.rs:836–841`). When the UI starts with `ZOETROPE_STATE_DIR=state` outside the agent’s project, its generated `--root 'state'` command resolves under the agent’s project and reads/writes another store. A standalone public `FleetConsole` probe (`/tmp/zoetrope-semantic-relative-root.rs`) confirmed the saved draft targets `agent-project/state` instead of the UI’s `state`. Resolve the root to an absolute path before sharing it with workers and tracking drafts.

Both probes used disposable local fixtures and unavailable runtime sockets; no native session was controlled.

### Final resolution and verdict — 27 October 2026

Focused follow-ups reviewed `ec35ad3...a63850a` and `a63850a...856c565`, including integrated SEM-4 documentation and review-fix evidence. Final reviewed HEAD is `856c5654020a2829f4d3fb3214a7fa1660b8225a`.

- **Empty-plan finding resolved.** `src/provider/work.rs:89–107` imports empty revisions. Per-producer replacement removes withdrawn entries, and `src/fleet/work_view.rs:88` keeps expired selection unavailable until deliberate reselection. The public withdrawal regression covers active, empty and replacement plans and no implicit retargeting. This satisfies spec line 45: “Native plan changes are revisions; title/index similarity does not prove durable identity.”

- **Relative-root finding resolved.** `FleetConsole::new` resolves the root once before workers start. `src/fleet/mod.rs:1652–1678` validates and quotes the exact absolute root for both tracking and criteria drafts. The public cross-project regression checks both read/apply paths and reads persisted state from another project directory. This satisfies spec line 50: “exact provider/session/state location”.

- **Source-launch follow-up clear.** `src/main.rs:338–343` supplies the running executable; the shared command helper validates/caches its absolute path and quotes read/apply/help commands. The actual PTY regression launches with an incompatible `zoe` first on PATH, then executes both generated commands from another directory and confirms the current executable and exact store. Embedded callers retain their documented optional override/default.

- **Compact criteria reads clear.** `src/fleet/mod.rs:1765` now requests filtered `context`. The public privacy regression executes that drafted command with saved secret answers, drafts and outbox data present, confirming correct goal, excluded private payloads and the 64 KiB bound. This matches spec line 44: “excluding feedback/questions/delivery history”.

The plan-status/current-entry refactor preserves unknown status honesty and selection. Reporting, opt-in guidance, navigation and handoff align with implemented behavior; root is updating the guide’s supplied-executable substitution note and final validation record.

**Verdict: clear. Both original P2 findings are resolved; no remaining confirmed Spec defects, missing requirements or scope creep found.** Integrated evidence records 319 native tests and strict Rust 1.99/1.88 Clippy passing; the merger reran focused regressions. No installation, publication or native live action is claimed. Root owns final validation updates.


Final counts: Standards 0 remaining (originally 1 documented breach and 1 maintainability judgement); Spec 0 remaining (originally 2 P2 findings). The axes remain separate.
