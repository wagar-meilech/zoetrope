# Text readability independent review

Baseline `1173c674771cadae0c676728d63d02bf9c54f36f`; original feature tip
`59c514edceb50b96ee496dbb86775e8723aae185`. Each reviewer inspected the fixed
three-dot diff independently. No live native input was sent.

## Standards — original report

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

## Spec — original report

# Spec review — text readability

Pinned range: `1173c674771cadae0c676728d63d02bf9c54f36f...59c514edceb50b96ee496dbb86775e8723aae185` (one feature commit; nonempty diff). Reviewed against the user's readability request and `docs/TEXT-READABILITY.md`. Narrow reproductions used the real debug executable, isolated native fixtures and an owned tmux server; no live guidance or project edits.

- **P2 — Link parsing consumes unrelated literal brackets** — `src/fleet/rich_text.rs:207–224`. The scanner finds any later `](` after the first `[`, without requiring its matching closing bracket. Public text `Keep [raw] and read [the spec](docs/SPEC.md).` renders `Keep raw] and read [the spec (docs/SPEC.md).`, losing the literal opening bracket and combining unrelated text into the link label. The contract says “Malformed/unsupported markup retains literal text” and “Link targets stay visible.” Match the immediate balanced label/target, or preserve an unmatched bracket literally and continue scanning; do not consume across another bracket pair.

- **P2 — An unclosed message fence consumes subsequent UI sections** — `src/fleet/mod.rs:1750–1781`, `src/fleet/rich_text.rs:18–32`. Public narration and the generated command/provenance sections are concatenated into one Markdown input. With narration ending in an unclosed triple-backtick block, the next `## Latest shell command` and `Recorded:` lines render inside its code gutter. The command's generated closing fence closes the narration's block, so no unclosed-fence warning appears. The contract requires distinct visual elements and “code indentation and omission/fence diagnostics [to] make imperfect input inspectable.” Parse independent native-message/command fragments or close/report unfinished formatting at their typed boundary so source markup cannot absorb later sections.

Both reproduced in `/tmp/zoetrope-readability-spec-probe.py`; capture `/tmp/zoetrope-readability-spec-probe.txt`.

The soft-blue palette, path/code cues, selected-message bounds/cache, public-phase filtering and literal editor behavior otherwise match the inspected direction. No material scope creep identified. Final installation/resource/PR completion remains root integration work, not a confirmed implementation defect.

**Verdict: 2 open Spec findings; worst severity P2.**

## Additional root visual QA

Synthetic 150×42 Commands and 60×28 readable captures showed two adjacent issues:
compact fields were clipped before parsing markup (dangling emphasis/link markers
and leaking multiline context), and broad slash-token recognition colored keyboard
hints/function-call prefixes as paths. The same isolated fix implementer owns
public rendering regressions for these alongside both confirmed Spec findings.

## Final rechecks

Pending isolated fix integration and independent rechecks. Original reports above
remain historical after resolution; final verdicts are recorded separately.
