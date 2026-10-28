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
| TXT-4 | Resolve review and narrow-layout edge cases | done | /root/text_review_fixes | TXT-2, TXT-3 | Integrated second fix; independent Standards and Spec rechecks clear. |
| TXT-5 | Verify, install and publish the final update | done | main | TXT-4 | Final checks/reviews passed; installed viewer refreshed; PR updated. |

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

## Final validation — 28 October 2026

Application source `8a9e82a2adecdbd08f7a086d05b1b4e06354a831` is independently
rechecked and integrated. Handoff `5d562c3` retains identical app/test/package
inputs; subsequent changes are completion documentation and synthetic artifacts.

| Check | Result |
| --- | --- |
| Full native all-feature suite | **386 passed**, one manual resource probe ignored in the ordinary run |
| Explicit manual 1,000/4,000-goal history probe | Passed; startup 315/836 ms, peak UI poll 19.705/82.379 ms |
| Portable suite | **189 passed** (176 library +13 Console) |
| Strict all-target/all-feature Clippy, Rust 1.99 and declared 1.88 minimum | Passed at the final fix source |
| Locked wasm frontend | Passed |
| Rust documentation, warnings denied | Passed |
| Standalone crate package | Packaged/compiled; nothing uploaded |
| Formatting / diff hygiene | Passed |
| Independent Standards and Spec reviews | Both clear after rechecks |
| Actual release / updater / installed skill | Built and installed; binary hash and packaged skill verified |

Portable/wasm checks at `59c514e` cover the unchanged shared/provider source;
later fixes only change native presentation/tests. Final native/rustdoc/package/
release checks ran after final fix integration. The targeted final fix suites
also passed 104 tests (15 Console, 8 runtime, 81 UI) before the final combined run.
There are no hosted CI checks reported for this branch; these are local results.

The release is 8,454,416 bytes, SHA-256
`957c58af61e4455be44b2c261c2ba38a2504a0545f877a882e1cb03cb2e13065`.
No Cargo manifest/lockfile or dependency changed in this continuation.

### Visual and resource evidence

[Actual terminal previews](research/text-readability-captures/README.md) include
150×42, 106×48 and 60×28 Work/readable, overview and Commands views. Final wide
Work/Commands and narrow readable views were visually inspected; public render
regressions cover the other layout/formatting cases. ANSI captures preserve the
actual soft-blue/teal/lavender styles under inherited NO_COLOR with explicit override.
All committed screenshots use synthetic data.

Two ten-second idle release samples followed three seconds of warmup:

| Fixture | RSS / peak RSS | Sampled CPU, one core | Idle terminal output |
| --- | --- | --- | --- |
| 12 tasks, 8 edges; Commands open with 6,719-byte structured public message | 8,796 KiB (8.6 MiB) | 0.1% | 0 bytes |
| 512 tasks,508 edges; Work graph | 18,816 KiB (18.4 MiB) | 0.2% | 0 bytes |

The samples ran concurrently with a short synthetic capture during warmup.
They do not measure arbitrary busy native sessions or guarantee platform-wide
performance. [Raw observations](research/text-readability-resource-results.json)
and the [public resource probe](research/probes/work-graph-resource.py) record
source sizes, release identity and practical limits. Add `rich commands` after
`12` to reproduce the structured-message sample.

The [public malformed-link probe](research/probes/text-readability-link-cost.rs)
uses equal-size criteria fixtures. Final release draws for 4/8/16/32 KiB malformed
inputs were 2.321/3.264/5.747/8.606 ms; controls were 2.524/3.072/5.245/7.790 ms.
The preceding quadratic version took 62.6 ms at 32 KiB in the reviewer's release
sample. Separate debug RED/GREEN sweeps and the one-pass implementation rationale
are recorded in [fix evidence](research/text-readability-fixes.md). These are
individual observations with scheduling noise, not asserted timing guarantees.

### Installed viewer and publication

The actual `zoe-update` rebuilt/reinstalled the same permanent checkout and user
tracking skill without warnings. `~/.local/bin/zoe` matches the checked release;
ordinary launch needs no Cargo command. Existing drafts/records remain intact.
The refreshed `readable` window in `zoe-work-graph` observes the same exact project
and coordinator through read-only navigation, using explicit color. It sends no
guidance/queue/stop and changes no project instructions. Raw live captures remain
local under `.local/`; the checked-in previews are synthetic.

[Issue #22](https://github.com/wagar-meilech/zoetrope/issues/22) closes through
[PR #8](https://github.com/wagar-meilech/zoetrope/pull/8) when merged. The PR is
updated and ready for review; main remains unmerged. The [review history](research/text-readability-review.md)
preserves each independent axis and resolution separately.
