# Text readability review fixes

Isolated worktree `/tmp/zoetrope-text-fixes`, branch
`fix/text-readability-review`, based on application/integration `59c514e`.
The existing public console input/render and executable/files/process seams were
already authorized. No private parser tests, global installation, native live
input, remote publication or main merge were performed.

## Confirmed Spec findings

1. **Literal bracket/link matching.** Inline links match the immediate label
   delimiter and a balanced target; an unrelated or unmatched opening bracket
   remains literal and does not prevent a later valid link. Parenthesized target
   paths such as `docs/(draft)/SPEC.md` stay visible and colored.
2. **Independent activity sections.** Native narration, generated provenance,
   literal shell input and pane hints use typed fragments. Markdown fence state
   ends, with an unclosed-fence diagnostic when needed, at its own fragment.
   A recorded `## Latest shell command` inside a valid code block remains code;
   it cannot be mistaken for an actual section delimiter. Shell input renders
   directly as literal code, without round-tripping through generated fences.

The renderer shares its 32 KiB / 512-source-line budget across every fragment;
formatting state resets do not reset resource limits. It remains cached/pure.

## Root visual QA findings

- Compact priority fields parse/style their bounded first line before clipping
  to terminal width. Clipping retains styles and whole graphemes, marks omissions
  with an ellipsis, and keeps later field paragraphs out of the compact area.
  Full recorded context remains below. Availability rows remain visible at 60×28.
- Path tokens separate punctuation/code from recognizable paths. Explicit absolute/
  relative paths and recognizable file suffixes receive teal. Ambiguous ordinary
  slash-separated phrases such as `PageUp/PageDown` remain ordinary text. In
  `read_source("src/parser.rs");`, only the path is teal; function/quotes keep code
  styling. Recognition performs no filesystem lookup.

## Vertical RED/GREEN

Each command used `CARGO_TARGET_DIR=/tmp/zoetrope-text-build/fixes`:

```sh
cargo test --offline --locked --test fleet_ui literal_brackets_do_not_consume_later_links_or_balanced_path_parentheses -- --exact
cargo test --offline --locked --test fleet_ui recorded_message_fences_cannot_absorb_commands_or_pane_footers -- --exact
cargo test --offline --locked --test fleet_ui narrow_context_formats_before_clipping_and_keeps_multiline_detail_below -- --exact
cargo test --offline --locked --test fleet_ui path_highlighting_keeps_navigation_words_and_surrounding_code_distinct -- --exact
```

Each test failed on its reported behavior before its implementation change and
passed afterward. The fence regression exercises both closed and unclosed native
messages in Fleet, Agents and Commands; real command headings retain their style,
source content/footers remain inspectable, and recordings are unchanged.

The additional public regression
`independently_formatted_activity_sections_share_the_pane_display_limit` supplies
512 native message lines plus later sections. It confirms one visible pane-limit
notice and omission of later command text rather than a per-fragment budget reset.
It was added after the fragment slice and passed; no separate RED is claimed.

Raw local logs are under `/tmp/zoetrope-text-build/` (`link-*`, `fragments-*`,
`compact-*`, `paths-*`, `fragment-bound`, `public-regressions`, `clippy-*`).

## Checks

- Combined public Console/UI/runtime suites passed: 15 Console, 8 PTY/runtime and 77
  UI tests, with the ordinary manual resource probe ignored. The subsequently
  added shared-budget test passed separately.
- Strict all-target/all-feature Clippy passed on Rust 1.99 and the declared 1.88
  minimum. The 1.88 run reused `/tmp/zoetrope-wg-build/sources-msrv`; Rust 1.99 used
  the isolated fixes target directory.
- Formatting and `git diff --check` passed.

Root integration owns final full-suite/resource/terminal checks, independent
rechecks, installed app/skill update, issue/PR publication and worktree cleanup.

## Merger integration

Clean handoff `21139996c14a3bc6cd796ad5685be460612fe11f` (application fix `f12885b`) was merged
conflict-free into `feat/goal-console` as `ebbed25d7df9a532d4327fa4f6fa8b3686ee7d23`. The merged tree exactly
matches the handoff before these documentation-only completion notes. Application,
public tests and package inputs therefore retain the implementer's checked source.
The coordinator's untracked synthetic-preview index was preserved unchanged.
TXT-4 is integrated/done; TXT-5 remains active for root's final combined checks,
independent rechecks, installation and publication. No broad tests, install,
publication or native input were performed by this merger.

## Follow-up recheck fixes

A second independent recheck found two remaining issues. Work fields and some
other generated read-only panels still shared Markdown state across externally
supplied values; malformed link targets also caused overlapping tail scans.

Work context, labels, provenance and generated controls now use independent dense
field/row fragments. The same seam covers criteria/evidence, plan steps, question
headers/bodies/options/descriptions/answers, document names/paths and delivery
snippets. Normal adjacent fields add no blank rows. Native source fields retain
their original stored values and editors remain literal. The shared pane budget
still applies across every fragment.

The public Work-pane regression
`malformed_work_fields_cannot_absorb_later_fields_provenance_or_controls` was RED
when an unfinished task label/Purpose/Source fence styled later fields as code;
it is GREEN with separate Status, Next, Provenance, Recorded and Controls plus
unchanged stored work. The cross-panel regression
`malformed_read_only_panel_values_cannot_absorb_later_rows_or_hints` was RED on a
criterion fence absorbing the next criterion. It is GREEN across criteria,
questions, documents and deliveries, including unchanged recorded values.

The link matcher now computes parenthesis widths once per inline input and then
looks up each target end in constant time. A u16 width table and opening stack are
bounded by the existing 32 KiB pane input; zero marks unmatched openings. Literal
bracket fallback and balanced-parenthesis targets keep their existing behavior.
The new public regression
`many_incomplete_link_targets_keep_literal_text_and_a_later_valid_link` confirms
literal recovery and a later valid balanced link after 1,024 incomplete candidates.

The deterministic public render probe is committed at
[probes/text-readability-link-cost.rs](probes/text-readability-link-cost.rs).
It creates owned recordings/state, renders real criteria through FleetConsole,
prints equal-size punctuation controls and malformed targets, then cleans up.
No private lexer calls or brittle timing threshold are used. Build against the
native libzoetrope/ratatui/crossterm artifacts, as in the root resource probes.
Debug RED/GREEN observations are retained under `/tmp/zoetrope-text-build/` as
`link-cost-red.log` and `link-cost-green.log`. They are individual fixture samples,
not timing guarantees. Final measurements and check outcomes follow below.

## Second fix integration

Immutable source `8a9e82a2adecdbd08f7a086d05b1b4e06354a831` was merged conflict-free
into `feat/goal-console` as `465604f897cf61de7d06c1a2c8bbb963f10fb93f`. Application,
public tests and package inputs exactly match that independently rechecked source.
Only completion documentation changes follow this merge; the coordinator-owned
untracked preview index was preserved unchanged. TXT-4 is done and TXT-5 remains
active. This merger ran no tests, installation, publication or native input.
