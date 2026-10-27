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

**At the original review, WG-6 integration was pending, not an implementation defect:** root's final combined checks/resource evidence, review fixes/rechecks, live app/skill installation, read-only live observation, and final PR/status documentation remain integration work. Existing synthetic terminal captures support readability claims without establishing every terminal/runtime.

## Integration check finding

At `1478ba6`, the full native suite passed 370 tests (one existing manual probe
ignored), portable tests passed 176 library and 13 Console tests, and current
strict Clippy, formatting, WebAssembly, rustdoc and standalone packaging passed.
Strict Clippy on Rust 1.88 found one nonminimal boolean expression in color
configuration. That compatibility check is included in the same isolated fix
worktree as the review findings; no lint suppression is planned.

## Standards — final recheck

Immutable fix tip: `2a65e3155310768a34833a4f0b762c4e6afc72df`.

- **P2 resolved:** `src/tracking.rs:738–771` now checks regular-file metadata before opening, uses Unix `O_NONBLOCK`, and validates the opened descriptor before bounded reads. The optional Unix-only native dependency preserves the portable boundary.
- **Optional smell resolved:** work/runtime cues now dispatch through `WorkStatus`/`RuntimeStatus`; presentation labels remain separate.
- Duplicate-ticket namespace rollback and exact-goal contention fixtures introduce no additional standards findings.

An independent disposable probe confirmed initial FIFO rejection, regular-source recovery, replacement-FIFO rejection with retained last-good count/fingerprint, and usable status/disable. Each command returned in 15–23 ms. Changed hunks and recorded validation were reviewed; broad suites were not repeated.

**Verdict: no open standards findings at the fix tip.** Original observations above are preserved as historical findings.

## Final installer recheck

Installer commit `f5e1d29ccd31d335ab4cc85f052a07fa8cceb813`, integrated at `2b06e537c4c5256130380dea398893b9ef71beb1`: **no additional standards findings**. `copy_additions` uses portable shell/globs and `cp -R -P`, recurses only through real directories, preserves nonconflicting hidden/nested additions, and gives packaged paths precedence without following old links. The public regression exercises two updates, both conflict shapes and unchanged external link targets. No installation or broad rerun performed. The application verdict at `2a65e315` remains unchanged.

### Standards — final installer recheck

Installer commit `f5e1d29ccd31d335ab4cc85f052a07fa8cceb813`, integrated at `2b06e537c4c5256130380dea398893b9ef71beb1`: **no additional standards findings**. `copy_additions` uses portable shell/globs and `cp -R -P`, recurses only through real directories, preserves nonconflicting hidden/nested additions, and gives packaged paths precedence without following old links. The public regression exercises two updates, both conflict shapes and unchanged external link targets. No installation or broad rerun performed. The application verdict at `2a65e315` remains unchanged.

## Spec — final recheck

Spec review and fix recheck

Original comparison: `7c17a367c40e09842139ec40b91f89fc94fdfb89...4681c772fc2294a3fd2efaf2c1eff1d5bce188d8`. Final immutable fix tip: `2a65e3155310768a34833a4f0b762c4e6afc72df`, inspected in `/tmp/zoetrope-wg-fixes`.

Reviewed the requested diff/log against the answered questionnaire, approved spec/plan and WG-1–6 tickets; inspected implementation, fixtures, skill guidance and terminal captures. Final recheck read `docs/research/work-graph-review-fixes.md` and reproduced both findings through `/tmp/zoetrope-wg-build/fixes/debug/zoe`, without broad test reruns.

- **Original P2, resolved — Nonregular source blocks monitoring** — original `src/tracking.rs:751–753` opened a FIFO before regular-file validation, hanging refresh while holding the tracking lock. Spec `docs/WORK-GRAPH-SPEC.md:78`: “Import bounded, explicitly allowlisted source changes”; line 79 requires invalid input to retain last-good data “with a stale/error explanation.” The fix prechecks metadata, uses Unix nonblocking open, and validates the opened descriptor before reading. Independent disposable recheck rejected an initial FIFO and a regular-file replacement in 0.022/0.014 seconds. Last-good identity, status, owner and provenance remained intact with stale diagnostics; status and disable remained responsive. The open-time replacement race is addressed by inspected descriptor/nonblocking logic, not a claimed deterministic race test.

- **Original P2, resolved — Duplicate selected ticket IDs accepted** — original `src/tracking.rs:891–900` accepted two selected ticket files both declaring `ID: T1` without errors. Spec `docs/WORK-GRAPH-SPEC.md:77`: “Duplicate IDs are invalid.” The fix validates the shared ticket namespace before publishing and restores previous tasks, fingerprints, retirement and field authority on collision. Independent recheck confirmed initial duplicates yield stale errors and zero current source tasks; a colliding edit preserves the coherent last-good namespace, correction recovers, and independently namespaced tables can reuse T1 without merging tasks. Fixture: `/tmp/zoe-spec-recheck-pxrk70n3`.

**Final verdict:** no unresolved confirmed Spec findings in this reviewed scope. Adjacent typed visual cues, color/MSRV adjustment and exact-goal lock fixtures introduce no identified requirement regression. No material scope creep found.

## Integration completion

The final native suite passes 373 tests, with the existing manual resource probe
run separately. App/skill installation, update, terminal captures and read-only
live observation are complete. The [final validation](../WORK-GRAPH-VALIDATION.md)
records exact source boundaries, resources and practical limits. See
[review fixes](work-graph-review-fixes.md) for the RED/GREEN evidence and portable
installer follow-up.

Open confirmed findings: **Standards 0; Spec 0**. Neither axis has an open severity.
