# WG-1 implementation — 27 October 2026

Approved seams: console input/render/runtime messages and executable/external
files. No private parser, graph, or layout helpers are tested. Base: `dd7c744`,
branch `feat/work-canvas`, isolated worktree `/tmp/zoetrope-wg1`. Uses the TDD
skill and its tests/mocking references. No live project or native input changed.

## Vertical RED / GREEN evidence

Every command below used `CARGO_TARGET_DIR=/tmp/zoetrope-wg-build/canvas` and
`cargo test --locked --test fleet_ui <test-name> -- --exact`.

1. `work_opens_task_canvas_and_back_restores_the_fleet_view`: RED confirmed
   `w` retained Fleet nodes. GREEN creates a separate cached rataflow canvas,
   visible project/goal breadcrumb and back action; Enter opens Work, Esc restores
   Fleet's camera and selection.
2. `work_canvas_draws_declared_prerequisites_and_keeps_parent_grouping_separate`:
   RED had no drawn dependency. GREEN draws labeled prerequisite-to-task edges,
   places disconnected tasks, and identifies parent links as grouping only.
3. `cyclic_missing_and_disconnected_work_remain_visible_and_selectable`: RED
   lacked cycle explanation. GREEN identifies strongly connected components,
   displays unresolved prerequisite cards, and keeps independent work selectable.
4. `task_mouse_selection_and_dragged_camera_survive_status_refresh`: RED exposed
   stale pre-centering pixels and reset dragged positions on status refresh.
   GREEN clears the intermediate frame and retains placement, selected task,
   exact reported owner and pan across refresh. Commands and Replay routes remain
   reachable. Native control regressions use the explicit Agents route (`a`)
   because Enter now opens Work.
5. `agents_back_returns_to_the_selected_work_and_its_camera`: RED lacked distinct
   view labeling and return history. GREEN labels Fleet/Work/Agents and restores
   the preceding task selection and camera via keyboard or visible back action.
6. `dense_work_graph_reports_omissions_and_navigation_stays_bounded`: RED hid
   reference omissions beyond the canvas width. GREEN prioritizes the limit
   notice and caps rendering at 512 task nodes, 64 unresolved placeholders and
   2,048 prerequisite edges. SCC analysis considers declared links, including
   those omitted from drawing. A public 80-task/5,056-reference fixture navigates
   to its last task within the conservative three-second interaction budget.

The graph uses cached WorkRecord projections and bounded O(V+E) layering/SCC
work only for topology changes. Ordinary status/context updates preserve task
placement. Parent links never create prerequisites; native plan order never
creates prerequisites. Separate Fleet and Work flows retain independent cameras.

## Checks

- Existing public fleet tests passed after the first five slices: 49 passed,
  1 existing manual resource probe ignored.
- Strict `cargo clippy --locked --all-targets -- -D warnings` passed.
- Further source integration and final checks are recorded below.

7. `bound_source_tasks_draw_dependencies_and_human_ownership_never_selects_a_recipient`:
   after the parent's authorized merge of immutable WG-3 interface `c1c3f07`,
   RED confirmed source tasks were invisible. GREEN imports a real designated
   Markdown table through the executable, draws its declared dependency, retains
   source/task identity and shows Alice's human ownership without selecting a
   native recipient. Unknown/unassigned source owners never fall back to main.

8. `source_snapshot_replaces_the_old_plan_and_removed_task_selection_expires`:
   after the parent's authorized merge of WG-3 `55c0115`, RED showed the old
   standalone report alongside the source plan. GREEN honors the canonical
   source snapshot, including valid empty sources: retiring a selected source
   task expires its recipient and never resurrects old explicit/native work.
9. `work_details_prioritize_task_context_and_keep_full_commands_behind_commands`:
   RED showed full public shell commands in default Work details. GREEN places
   purpose/next/blocker before source metadata and exposes full activity only
   through the explicit Commands route.

10. `readable_work_view_restores_the_selected_task_after_panning_offscreen`:
    RED showed `r` only recentered Fleet's camera. GREEN restores the selected
    task in the active Work canvas. The existing orphan-agent/search regression
    also exposed a breadcrumb hiding search state; the header now shows the
    current search while retaining its view label.
11. `reordering_source_rows_preserves_task_identity_placement_and_camera`:
    RED moved stable tasks after a table row reorder. GREEN compares topology
    as IDs and prerequisite sets, preserving placement/camera for row order and
    ordinary context updates. The test waits for the changed context to appear
    through normal public background refresh.
12. `a_resolved_logical_report_suppresses_its_owners_native_plan_fallback`:
    after merging the latest integration/WG-2 via `3319deb`, RED duplicated an
    explicit alias report and the same native owner's fallback plan. GREEN uses
    the scoped public owner mapping for fallback suppression as well as steering.

## Integration and final validation

- Preserved WG-3 ancestry via merges of the parent's explicitly authorized green
  interface commits `c1c3f07` and `55c0115`; later source hardening remains WG-3.
- Merged current `feat/goal-console` (`05938ec`) via `3319deb`, including WG-2.
  The only conflict was appended public tests; both sets were retained.
- Full public fleet/control suite: `cargo test --locked --test fleet_ui`.
- Strict lint: `cargo clippy --locked --all-targets -- -D warnings`.
- Formatting: `cargo fmt --all -- --check`.
- All Cargo commands use the isolated target directory above. Root owns broader
  native/portable/browser/MSRV validation and final combined review.

WG-5 retains the planned status palette, semantic overview/progress styling and
broader compact-layout polish. No new dependencies, source discovery/network
polling, live-project edits or native input were introduced by the canvas.

Final results on the integrated WG-1 tree: **59 fleet/control tests passed, 1
existing manual probe ignored**, in 10.45 seconds. Strict all-target Clippy,
formatting check and `git diff --check` all passed. The dense 80-task graph test
passed with its interaction budget; its full fixture/test invocation took about
1.4 seconds in the isolated debug build. No broader root checks were duplicated.
