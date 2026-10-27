# Work graph independent review

Initial immutable feature tip: `4681c772fc2294a3fd2efaf2c1eff1d5bce188d8`.
Baseline: `7c17a367c40e09842139ec40b91f89fc94fdfb89`.
Integration `1478ba6` preserves identical application/test/skill source.

Two independent reviewers inspected the three-dot diff and approved specification.
The reports below remain separate; the FIFO finding appears on both axes.
Fixes and final rechecks are recorded after these original findings.

## Standards — original report

Range: `7c17a367c40e09842139ec40b91f89fc94fdfb89...4681c772fc2294a3fd2efaf2c1eff1d5bce188d8`.

## Documented-standard breach

- **[P2] Reject nonregular source files before opening them** — `src/tracking.rs:735–753`, especially `File::open` at line 751. `CLAUDE.md:30` requires bounded local IO/resource behavior; `docs/WORK-GRAPH-SPEC.md:78` requires bounded allowlisted import. The code obtains metadata but checks `is_file()` only after opening. A selected `WORK.md` FIFO blocks indefinitely in `open`, before the regular-file diagnostic. This also affects an existing source replaced with a FIFO: inventory refresh stalls while holding `tracking.lock`, preventing later refreshes and blocking tracking status/disable. A disposable CLI probe using `/tmp/zoetrope-wg-build/visuals/debug/zoe` and a temporary named pipe timed out after two seconds and was killed. Reject the already-observed nonregular type before opening, and use a nonblocking open/descriptor validation to keep replacement races from blocking.

## Useful smell, not a documented violation

- **Primitive Obsession / Repeated Switches** — `src/fleet/visual.rs:12–48`, with callers in `work_view.rs:235–272` and `visual.rs:79–89`. `Cue::work(&str)` dispatches on presentation phrases for both work and native runtime state. Callers repeat strings such as `"in progress"` and `"reported done"`; changing wording silently changes status cues to unknown. Prefer the existing `WorkStatus`/runtime status types for semantic dispatch and format their labels separately. This is a maintenance suggestion, not a required architecture rewrite.

No other documented-standard breach found in the reviewed provider/native boundary, cached owner joins, source precedence, installer, or skill workflow. Native transcript formats stay behind `src/provider`; the new filesystem module is native-gated; rendering introduces no IO; the skill uses conditional reference pointers and preserves exact-goal/local-disable semantics. No broad test suite or native/live setup was run.

## Spec — original report

Spec review: fixed baseline `7c17a367c40e09842139ec40b91f89fc94fdfb89` → immutable tip `4681c772fc2294a3fd2efaf2c1eff1d5bce188d8`. Reviewed the requested three-dot diff and commit log against the answered questionnaire, approved spec/plan and WG-1–6 tickets. Inspected code, fixtures, skill guidance, and actual terminal captures; used disposable public CLI reproductions.

- **P2 — Nonregular sources can block monitoring indefinitely** — `src/tracking.rs:751–753`. The importer calls blocking `File::open` before checking `is_file()`. After a valid source is replaced by a FIFO named `WORK.md`, `tracking sync` hangs opening it; the automatic inventory refresh follows the same path while holding the exclusive tracking lock. This prevents inventory refresh and other tracking commands instead of showing stale last-good data. Reproduced with the integrated visuals executable and a 1.5-second subprocess timeout; fixture `/tmp/zoe-spec-review-_53ye7y8`. Spec `docs/WORK-GRAPH-SPEC.md:78`: “Import bounded, explicitly allowlisted source changes”; line 79: “Invalid, oversized or temporarily unavailable input instead retains last-good data with a stale/error explanation.” Reject nonregular sources before a potentially blocking open, including replacement races.

- **P2 — Duplicate IDs across the selected ticket set are silently accepted** — `src/tracking.rs:891–900`. Each `@ticket` file contains one row, so the per-source duplicate check cannot reject two selected files both declaring `ID: T1`. Both become current fresh tasks, with `error:null`, `stale:false`, and no source diagnostic; ambiguity is noticed only if some task references T1. Reproduced using `T1.md@ticket` and `T1-copy.md@ticket`, fixture `/tmp/zoe-spec-review-tickets-4uo_vwqc`. This inflates reported-work counts and leaves the shared ticket dependency namespace ambiguous. Spec `docs/WORK-GRAPH-SPEC.md:77`: “Duplicate IDs are invalid.” Validate uniqueness across the explicitly selected ticket namespace and retain last-good data on conflicting edits; independently namespaced table sources need not collide.

No additional material scope creep found. The Rust/local importer, fallback, reusable skill, graph navigation, progress separation and color controls substantially match the approved direction.

**Pending WG-6 integration, not implementation defects:** root's final combined checks/resource evidence, review fixes/rechecks, live app/skill installation, read-only live observation, and final PR/status documentation remain integration work. Existing synthetic terminal captures support readability claims without establishing every terminal/runtime.

## Integration check finding

At `1478ba6`, the full native suite passed 370 tests (one existing manual probe
ignored), portable tests passed 176 library and 13 Console tests, and current
strict Clippy, formatting, WebAssembly, rustdoc and standalone packaging passed.
Strict Clippy on Rust 1.88 found one nonminimal boolean expression in color
configuration. That compatibility check is included in the same isolated fix
worktree as the review findings; no lint suppression is planned.

## Fixes and rechecks

Pending the isolated review-fix handoff. See
[work-graph-review-fixes.md](work-graph-review-fixes.md) for RED/GREEN and checks
when integrated. Installation and final source-specific resource observations
follow resolved findings; the existing live viewer is unchanged meanwhile.
