# WG-5 implementation — 27 October 2026

Approved seams: public FleetConsole input/render and runtime messages, plus the
real executable in a disposable terminal. Uses the TDD skill and its tests and
mocking references. Base `55d35bb`, branch `feat/work-visuals`, worktree
`/tmp/zoetrope-wg5`. No private layout/parser tests, native session input, live
project changes, environment mutation, network polling or new runtime dependency.

## Vertical RED / GREEN evidence

All Cargo commands use `CARGO_TARGET_DIR=/tmp/zoetrope-wg-build/visuals`.
Each slice ran the named public test before implementing its behavior, then ran
the same test to GREEN. UI invocation:
`cargo test --locked --test fleet_ui <test-name> -- --exact`.

1. `work_statuses_have_consistent_color_and_non_color_cues`: RED lacked semantic
   task titles. GREEN adds a shared palette, glyphs and visible legend: planned
   blue, active cyan, blocked red, reported done green, cancelled gray and unknown
   magenta. Fleet project/goal/agent cards also have distinct glyphs and live
   status cues. The selected border, panel borders and actions share gold.
2. `overview_retains_status_and_selection_before_readable_zoom_restores_titles`:
   RED reduced cards to empty borders and repeated overlapping `requires` text.
   GREEN uses compact status cells with a reverse/bold selected cue and explicit
   selected marker. Edge labels disappear at small zoom; `r` restores the chosen
   full task card, with its title/status/owner/dependency context.
3. `reported_work_counts_keep_cancellation_unknown_and_verified_outcomes_separate`:
   RED had no reported-work summary. GREEN counts the canonical current work
   projection by done/active/blocked/planned/cancelled/unknown, separately from
   the explicitly labeled evidence-backed Outcomes gauge. Cancelled and unknown
   work never count as done. Source and recorded-time attribution remain visible.
4. `narrow_work_view_keeps_a_readable_task_card_beside_focused_context`: RED
   allocated only three usable graph rows and clipped capability labels. GREEN
   fits a full seven-row task card, a complete legend and compact inspector at
   60×28. Owner, purpose, next action, blocker and all three capability states
   take priority; full metadata and reasons remain available by PageDown. Full
   commands stay behind Commands. Only actual terminal resizing recenters;
   returning from Agents preserves the preceding Work camera and selection.
5. `color_modes_emit_real_terminal_ansi_and_global_flags_preserve_fleet_launch`:
   run with `cargo test --locked --test fleet_runtime <test-name> -- --exact`.
   RED rejected `--color`. GREEN strips global auto/always/never flags before
   choosing Fleet/replay/subcommands. Real PTY output contains color SGR for
   `always` despite inherited `NO_COLOR=1`; auto honors that value, never emits
   no color even without it, and ordinary auto emits color on the terminal.
   Bare `zoe --color always` remains Fleet. Crossterm's public atomic override
   affects only the current process; no environment variables are mutated.
6. `an_unknown_native_status_never_inherits_a_known_status_color_from_its_label`:
   RED colored an unrecognized native label `reported done` green. GREEN prefixes
   unknown status labels explicitly and keeps the unknown glyph/color/count.
7. `focused_work_marks_selection_without_color_and_styles_its_blocker_consistently`:
   RED had unstyled status/blocker details. GREEN shares status colors with the
   inspector, emphasizes useful field labels and labels full-card selection
   `› selected`, so meaning survives monochrome output.
8. `opening_work_from_search_keeps_its_breadcrumb_and_visible_back_action`:
   RED showed only Search, hiding the Work breadcrumb/back label. GREEN retains
   the view/back breadcrumb alongside search context, with clickable return.
9. `narrow_graph_limit_notices_keep_the_status_legend_visible`: RED let wrapped
   limit/cycle/unresolved summaries displace `? unknown`. GREEN reserves rows for
   the complete legend independently of the bounded summary/omission notice.

10. `tall_work_overview_retains_meaningful_labels_across_a_twelve_task_branching_plan`:
    RED showed 3–7-character title fragments across a mostly empty tall canvas.
    GREEN layers prerequisites vertically on tall canvases and retains horizontal
    flow on short canvases. Isolated work fills otherwise unused side slots
    without creating edges. A 12-task fixture at 106×48 keeps meaningful labels
    across both branches. Adaptation happens only on topology/actual terminal
    changes, retaining dragged positions and ordinary refresh camera stability.
    Compact cards prioritize title space; edge labels stay at reading zoom and
    larger prerequisite gaps keep arrows visible. Selected recorded times use
    a single UTC line. Actual terminal captures, then the root's visual review,
    confirmed the improvement.

The public `cyclic_missing_and_disconnected_work_remain_visible_and_selectable`
regression also exposed an `o` request being overridden during first draw/resize.
Its RED/GREEN fix retains explicit overview state while adapting, including when
fitting a small imperfect graph increases zoom above 1.0.

The public `agents_back_returns_to_the_selected_work_and_its_camera` regression
also caught an over-eager recenter after changing views. Its RED/GREEN fix keys
recentering to actual terminal dimensions rather than a changed panel layout.

## Real terminal visual QA

The reproducible [capture script](probes/wg-5-terminal-capture.py) launches the
real binary in an owned disposable tmux server, seeds its own local goal/reports
through the executable and captures both readable and overview views. The
fixture has a branching checkout workflow, 12 tasks including cancelled/unknown,
5 reported done and 0 of 5 verified outcomes. Its environment deliberately uses
`TERM=tmux-256color`, `NO_COLOR=1` and `--color always`. No keys are sent to the
live `zoe-live` session or any native agent.

Raw tmux ANSI/plain captures are paired with PNGs rendered from those captured
cells using Pillow/DejaVu Sans Mono. The PNGs are terminal-capture visualizations,
not browser mockups or evidence about every user's font/palette. Pillow and tmux
are QA-only dependencies. Capture dimensions were checked: 150×42, 106×48 and
60×28, each in readable and overview mode. All six were visually inspected.

The initial captures exposed dark blue/red contrast and a hidden Back action
when searching. Brighter blue/red shades and the retained breadcrumb corrected
those observations. Root visual review additionally identified the horizontal
chain's unused height and short labels. Adaptive vertical layering now fills the
tall overview with meaningful task labels and visible branching; root inspected
and accepted the improved 106×48 capture. The narrow capture retains a full task
card and focused context. Neither view exposes full command walls.

Reproduce after building the executable:

```bash
CARGO_TARGET_DIR=/tmp/zoetrope-wg-build/visuals cargo build --locked --bin zoe
python3 docs/research/probes/wg-5-terminal-capture.py \
  /tmp/zoetrope-wg-build/visuals/debug/zoe /tmp/zoetrope-wg5-qa
```

## Checks and integration

Before the final source integration: 66 public UI tests passed, one existing
manual resource probe ignored; all eight real-terminal/runtime tests passed.
Strict all-target Clippy passed. Final integrated checks and capture links follow.
