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
