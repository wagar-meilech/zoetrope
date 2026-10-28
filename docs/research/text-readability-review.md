# Text readability independent review

Baseline `1173c674771cadae0c676728d63d02bf9c54f36f`; final application source
`8a9e82a2adecdbd08f7a086d05b1b4e06354a831`. Reviewers inspected fixed three-dot
diffs independently. Original findings and intermediate verdicts below remain
source-specific history; the final rechecks take precedence.

## Standards

# Standards review — text readability

Reviewed immutable range `1173c674771cadae0c676728d63d02bf9c54f36f...59c514edceb50b96ee496dbb86775e8723aae185` and its single `feat(ui)` commit. Both refs resolve and the diff is nonempty.

Standards sources: `CLAUDE.md`, `docs/ARCHITECTURE.md`, `docs/DESIGN.md`, `docs/DISCOVERY.md`; no tracked `AGENTS.md`, `CONTRIBUTING.md` or `CODING_STANDARDS.md` was found.

## Documented-standard breaches

**None confirmed.**

- Provider-specific public-message extraction remains in `src/provider/`; the console receives attributed activity and indexed record positions without learning native JSON formats. Existing public-phase and stream/ancestry filtering still governs which message positions are retained.
- `Console::load_recorded_activity` runs on the inventory worker, with one selected-message cache. The old cached preview interface remains available. Selected reads are bounded to the indexed record, check regular-file metadata, use Unix nonblocking open and validate the descriptor before reading. Native-only code preserves the portable boundary and adds no dependency.
- The renderer performs no IO, source mutation, native action or model call. Its byte/line limits, UTF-8 boundary checks and literal control-character presentation bound and explain imperfect input. Status dispatch still uses `WorkStatus`; formatting does not update outcomes.
- Added tests use public Console/file and input/render seams, including actual styles, source preservation, drafts, cache refresh, UTF-8 bounds and private-content exclusion. No private tokenizer/layout tests were introduced. The implementation notes distinguish the four RED/GREEN slices from additional post-slice coverage.

## Baseline smells — judgement calls

No material smell merits a finding in this scope. The small display parser and selected-record loader keep formatting and native-format knowledge separate; a larger Markdown framework or per-history rich-message cache is not warranted by this request.

No broad suites, installation or live input were performed for this review. Root owns final terminal/resource/installation evidence.

**Verdict: Standards clear; 0 open findings and no open severity.**

## Standards — independent fix recheck

Rechecked immutable handoff `21139996c14a3bc6cd796ad5685be460612fe11f` against `59c514e`, including `docs/research/text-readability-fixes.md`. The typed fragments correctly isolate Markdown/shell state while sharing one pane budget. Post-format compact clipping retains styles/graphemes; source/provider separation and literal editors remain intact.

### Confirmed resource finding

- **[P2] Bound aggregate balanced-target scanning** — `src/fleet/rich_text.rs:305–315, 334–355`. The new `link()` loops through the entire remaining target looking for balanced parentheses. On failure, `inline()` advances just past one `[`, then tries every later opener. Repeated incomplete `[x](` targets therefore scan overlapping tails quadratically on the drawing thread. This conflicts with `CLAUDE.md:30`'s fast/resource-light requirement; the 32 KiB input cap does not cap cumulative matching work. A disposable public `FleetConsole` criteria/render probe measured 4/8/16/32 KiB malformed inputs at 33/117/464/1769 ms in the fixes debug build (same-size punctuation controls about 6–18 ms). The final release artifact measured 32 KiB at 62.6 ms versus 2.1 ms for its control. These are individual observations, not universal timings. Bound aggregate delimiter work or match targets in one pass, retaining literal fallback and valid later-link recovery.

Probe source: `/tmp/zoetrope-link-cost.rs`; release executable `/tmp/zoetrope-link-cost-release`. It uses only owned temporary native fixtures, public Store/Console input/render and an absent runtime socket. No private parser calls, native messages, installation or broad suites were involved.

**Verdict at `2113999`: 1 open Standards finding, worst P2.** The original clear verdict remains historical for `59c514e`; the quadratic scan was introduced by the fix.

## Standards — final recheck

Immutable second fix: `8a9e82a2adecdbd08f7a086d05b1b4e06354a831`.

**Quadratic matching P2 resolved.** `target_widths()` now builds balanced-parenthesis offsets once per inline input. Failed candidates use constant-time target lookup, while label scanning stops at the immediate next bracket. The existing 32 KiB byte cap makes the `u16` offset/width representation safe and bounds scratch memory; the aggregate pane byte/line budget still spans all fragments.

The same independent public debug-render probe returned malformed 4/8/16/32 KiB inputs in 5.6/8.3/24.8/21.1 ms; its 32 KiB control was 19.8 ms, versus the previous malformed 1,769 ms. These are individual fixture observations with scheduling noise, not timing guarantees. Source inspection confirms the overlapping-tail scan is removed. Root owns final release measurements.

Dense typed field/row fragments now isolate Work fields and generated criteria/evidence, question, document and delivery values without inserting blank rows between ordinary fields. Rendering remains pure, source text remains attributable, and recorded controls/provenance cannot inherit another value's fence state. Reviewed the changed hunks and public regression evidence; no additional documented-standard breach or material baseline smell was identified.

No broad suites, install, publication or live input were performed for this recheck.

**Final verdict at `8a9e82a`: Standards clear; 0 open findings and no open severity.** Prior findings/verdicts above are retained as dated source-specific history.

## Spec

# Spec review — text readability

Pinned range: `1173c674771cadae0c676728d63d02bf9c54f36f...59c514edceb50b96ee496dbb86775e8723aae185` (one feature commit; nonempty diff). Reviewed against the user's readability request and `docs/TEXT-READABILITY.md`. Narrow reproductions used the real debug executable, isolated native fixtures and an owned tmux server; no live guidance or project edits.

- **P2 — Link parsing consumes unrelated literal brackets** — `src/fleet/rich_text.rs:207–224`. The scanner finds any later `](` after the first `[`, without requiring its matching closing bracket. Public text `Keep [raw] and read [the spec](docs/SPEC.md).` renders `Keep raw] and read [the spec (docs/SPEC.md).`, losing the literal opening bracket and combining unrelated text into the link label. The contract says “Malformed/unsupported markup retains literal text” and “Link targets stay visible.” Match the immediate balanced label/target, or preserve an unmatched bracket literally and continue scanning; do not consume across another bracket pair.

- **P2 — An unclosed message fence consumes subsequent UI sections** — `src/fleet/mod.rs:1750–1781`, `src/fleet/rich_text.rs:18–32`. Public narration and the generated command/provenance sections are concatenated into one Markdown input. With narration ending in an unclosed triple-backtick block, the next `## Latest shell command` and `Recorded:` lines render inside its code gutter. The command's generated closing fence closes the narration's block, so no unclosed-fence warning appears. The contract requires distinct visual elements and “code indentation and omission/fence diagnostics [to] make imperfect input inspectable.” Parse independent native-message/command fragments or close/report unfinished formatting at their typed boundary so source markup cannot absorb later sections.

Both reproduced in `/tmp/zoetrope-readability-spec-probe.py`; capture `/tmp/zoetrope-readability-spec-probe.txt`.

The soft-blue palette, path/code cues, selected-message bounds/cache, public-phase filtering and literal editor behavior otherwise match the inspected direction. No material scope creep identified. Final installation/resource/PR completion remains root integration work, not a confirmed implementation defect.

**Verdict: 2 open Spec findings; worst severity P2.**

## Narrow fix recheck — 2113999

Immutable handoff `21139996c14a3bc6cd796ad5685be460612fe11f` (application fix `f12885b`), compared with original feature tip `59c514e`.

- **Original bracket P2 resolved:** immediate label matching and balanced target parentheses preserve unrelated/unmatched brackets while allowing a later valid link. The original public-executable/tmux reproduction now shows `Keep [raw] and read the spec (docs/SPEC.md).`.
- **Original native-message fence P2 resolved:** typed narration, provenance, literal shell and footer fragments end formatting independently. The original unclosed source now emits its diagnostic before a separately styled command heading/time. Recorded headings inside closed code are not treated as magic delimiters. Reproduction: `/tmp/zoetrope-readability-spec-recheck.py`, capture `.txt`.

**Remaining related P2 — Work-field Markdown still absorbs provenance/controls.** `src/fleet/work_view.rs` still assembles work fields, generated provenance and controls into one Markdown fragment. A valid report whose Purpose is `Read the code\n```rust\nUnclosed field code` makes the Work graph render `Next:`, `## Provenance`, `Recorded:` and `## Controls` inside that source code gutter. The requirement says “Task context, provenance and controls are visually separated,” and malformed input must remain inspectable. Reset source formatting at typed work-field/section boundaries too, preserving later fields and trusted sections independently. This is the same isolation class outside the now-correct native-activity fragments. Independent public reproduction: `/tmp/zoetrope-readability-context-recheck.py`, capture `.txt`; no live native input was sent.

Inspected the styled grapheme clipping, conservative path tokenization, and one shared 32 KiB/512-line budget across fragments, alongside the public regression coverage. No additional confirmed finding in those narrow changes; broad suites were not repeated.

**Recheck verdict: original 2 findings resolved; 1 related Spec P2 remains open.**

## Final narrow recheck — 8a9e82a

Immutable application fix: `8a9e82a2adecdbd08f7a086d05b1b4e06354a831`, clean isolated fixes worktree. Reviewed the follow-up diff and committed fix evidence; preserved all earlier findings above.

**Remaining work-field P2 resolved.** Dense typed field/row fragments isolate task labels, context values, source/provenance and generated control sections. Re-running the original public Work-graph fixture now ends the unfinished Purpose fence with its diagnostic, then renders `Next:`, `── Provenance`, `Recorded:` and `── Controls` independently. Source headings inside code remain literal instead of acting as reset markers. The native-message/link reproduction also still passes.

The same fragment handling covers criterion/evidence/plan values, question headers/bodies/options/descriptions/answers, document names/paths and delivery snippets. The exact public cross-panel regression was independently rerun against the pinned fixes worktree: **1 passed**. It verifies later rows/hints retain independent styles across criteria, questions, documents and deliveries, with stored values unchanged. No broad suite, installation or live native input was performed.

Inspected the parenthesis-width index and recovery regression: target matching is indexed once per bounded inline input, with bounded scratch space, while literal bracket fallback and balanced later links remain supported. One shared pane byte/line budget remains outside fragment iteration; state resets do not reset that budget. No additional confirmed Spec finding in this recheck. Final source-specific resource/install/publication checks remain coordinator integration work.

Public repro/captures: `/tmp/zoetrope-readability-context-recheck.py` / `.txt` and `/tmp/zoetrope-readability-spec-recheck.py` / `.txt`. Cross-panel run: `/tmp/zoetrope-readability-spec-crosspanel-recheck.log`.

**Final Spec verdict: all 3 confirmed findings resolved; 0 open findings, no open severity.**

## Root visual QA and completion

The same isolated implementer also resolved compact pre-format clipping,
multiline leakage and broad path-token coloring found in actual terminal previews.
Public regressions, source bounds and RED/GREEN evidence are in
[text-readability-fixes.md](text-readability-fixes.md). Final native tests,
resources and installation are in [the validation record](../TEXT-READABILITY.md).

Final open findings: **Standards 0; Spec 0**. Neither axis has an open severity.
