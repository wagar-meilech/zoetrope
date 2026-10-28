# Text readability follow-up

Requested by the user after the work-graph iteration: replace yellow headers and
make text readable through deterministic parsing, highlighting and visual groups.
The existing public console input/render and executable/process seams apply.
Tracked by [issue #22](https://github.com/wagar-meilech/zoetrope/issues/22).
Continue on `feat/goal-console` and PR #8; no merge into main.

## Working status

| ID | Task | Status | Owner | Depends on | Next |
| --- | --- | --- | --- | --- | --- |
| TXT-1 | Replace gold accents with soft blue | done | main | None | Rendering regression passed. |
| TXT-2 | Format structured text and highlight paths/code | done | main | TXT-1 | Public display/editor regressions passed. |
| TXT-3 | Preserve bounded selected-message structure lazily | done | main | None | Cache, UTF-8, refresh and privacy checks passed. |
| TXT-4 | Resolve review and narrow-layout edge cases | done | /root/text_review_fixes | TXT-2, TXT-3 | Integrated; public regressions and strict lint checks passed. |
| TXT-5 | Verify, install and publish the final update | active | main | TXT-4 | Run final combined checks/rechecks, install and publish. |

## Accepted behavior

- Soft blue replaces the gold header/border/selection accent, including the native
  replay banner. Existing status colors and non-color selection cues remain.
- Read-only detail panes recognize a small Markdown subset: headings, paragraphs,
  lists/checklists, quotes, emphasis, inline code, links and fenced code blocks.
  Paths/file references use teal; code uses lavender and framed blocks. Link
  targets stay visible. Diagnostic prefixes have warning/error styles.
- Task context, provenance and controls are visually separated. Narrow layouts
  keep compact priority fields rather than adding whitespace ahead of them.
- Malformed/unsupported markup retains literal text. Visible control characters,
  code indentation and omission/fence diagnostics make imperfect input inspectable.
  Formatting never runs a linter/command, rewrites a recording, opens a link, or
  infers task status/outcome verification. Editors retain literal text/cursor rules.
- Preserve selected public-message paragraphs instead of the old flattened
  240-character metadata preview. Fleet keeps that cheap preview; only the exact
  selected owner's message is read lazily off-thread and cached. Existing public
  phase/ancestry filters still exclude private/unknown/inherited content.
- Keep Rust and existing dependencies. Selected reads address the exact indexed
  record, at most the existing 64 KiB preview bound; structured message display
  caps at 8 KiB. One selected-message cache replaces an all-history rich-text cache.
  Display formatting caps input at 32 KiB / 512 lines with visible omissions.

## Verification and completion

Use the public console's actual rendered cells/styles, inputs and native fixtures;
no private tokenizer/parser tests. RED/GREEN logs are retained locally under
`.local/work-graph-validation/readability-*.log`. Final results and any review
findings/resolutions will be recorded here before installation/publication.

The original [work graph validation](WORK-GRAPH-VALIDATION.md) and synthetic
captures remain dated evidence of the preceding release.

## TDD evidence

Four vertical slices used `cargo test --offline --locked --test fleet_ui NAME -- --exact`:

1. `fleet_and_work_headers_use_a_readable_cool_accent`: RED observed indexed gold
   instead of the specified soft blue; GREEN checks Fleet/Work headers and border.
2. `work_context_separates_sections_and_highlights_paths_code_and_emphasis`: RED
   found unstyled paths; GREEN checks path/code/emphasis, sections, visible link
   targets and literal editor contents.
3. `recorded_messages_render_paragraphs_lists_and_code_without_rewriting_commands`:
   RED found an unstyled heading after correcting the fixture to its explicit public
   commentary channel. Initial implementation then exposed the old 240-character
   flattened preview. GREEN uses the lazy indexed-message read and checks paragraphs,
   lists, quotes, indentation, paths, code literals, diagnostics and unchanged source.
4. `recorded_control_characters_are_visible_without_changing_literal_code`: RED
   lacked visible tab/control formatting; GREEN checks visible escapes, UTF-8 paths,
   unmatched markup and literal backticks inside a recorded shell command.

The additional public Console regression
`selected_public_messages_preserve_bounded_structure_and_refresh_without_private_content`
checks compact previews, bounded UTF-8 structure, cache reuse while the file is
unavailable, append refresh after a multibyte record, exact goal isolation, private/
unknown phase exclusion, and Claude multi-part text with thinking excluded. It was
added after the parent preservation slice, not claimed as a separate RED/GREEN loop.

Initial combined checks pass: 378 native tests (one ordinary manual probe ignored),
189 portable tests, strict minimum-Rust Clippy, locked wasm, formatting and patch
hygiene. Final review, release/terminal/resource checks and installation follow.
