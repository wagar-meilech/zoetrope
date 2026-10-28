# SEM-3 — Work-first goal inspection

Implemented in `/tmp/zoetrope-sem-work-view`, branch `feat/semantic-work-view`, based on integrated `085ecde`. Uses the approved public Console/FleetConsole and executable/PTY seams. No model calls, real native-session mutation, external publication or push.

## Behavior and interfaces

- A selected goal shows meaningful work and its existing evidence-backed progress by default. Work is a selected-goal list, preserving compact fleet graph identities and layout rather than adding another portable graph abstraction.
- Explicit reports suppress native plan fallback for the same owner/producer. Reported status, owner, source/time, purpose/change/next/blocker, parent/dependencies, unresolved references and revision-qualified outcome links are visible. Unknown dependencies differ from explicitly empty dependencies; stale scope links are labeled. Reported done never verifies outcomes.
- `w` focuses current work. Once focused, arrows or `[` / `]` select entries; explicit keys retain identity across label/status/owner updates. Without work focus, arrows keep navigating the fleet graph. Main Enter/Space agent expansion remains compatible.
- Actual work ownership drives ordinary `f` steering through the existing `selected_agent()` / `control_target()` path. An open feedback editor retains its pinned `feedback_target` even when its work is reassigned. Unknown owners show useful short IDs with full IDs in details.
- Native plan selection is keyed by proposal identity plus step index; replacing a plan invalidates that selection and blocks implicit retargeting. `C` explicitly recovers to the exact visible goal's coordinator, even when the global graph already selected that Session ID or the prior plan disappeared. `T` remains explicitly coordinator-directed.
- Visible Work, Agents, Commands and Replay mouse/keyboard actions expose inspection routes. Agents opens the existing agent expansion. Commands reads the selected work owner's cached activity; Replay uses one request path for real mouse and keyboard input, preserving the existing read-only runtime and draft state.
- Native `PlanProposal.recorded_at: Option<String>` is backwards compatible and populated only from valid recorded timestamps. Compact CLI plan context includes that field; old plans show unknown time.
- Provider public activity now carries separate public-message source/time and shell-command time. A new undated message resets its message timestamp to unknown without changing a command's timestamp. Cards label their activity as recorded; private reasoning remains outside this projection.
- `Console::recorded_activity(&AgentKey)` returns one bounded cached `RecordedActivity` projection. The inventory worker copies only the requested owner's data, with a bounded retried request; draw performs no IO and does not duplicate all fleet activity. Existing provider stream filtering still determines which records belong to an agent.
- Narrow Work inspection gives the graph five rows, marks the current selection and prioritizes Owner/Purpose/Next plus separate control availability before scrollable full detail. Full work projection caps at 512 entries/plan sources, shows four nearby entries, and bounds displayed context/links. Limit notices make omitted detail explicit.
- Existing worker-side search now includes explicit work IDs/labels/context/owner IDs and native plan text. Search keeps current selection pinned.
- A tracking draft for a goal without criteria asks for concise editable `set_goal` outcomes including tests/final checks, kept unverified until evidence. It remains a local editable draft until the operator sends it.

## Vertical TDD evidence

Each row's focused command was run RED for the listed missing behavior, then GREEN. Commands use `CARGO_TARGET_DIR=/tmp/zoetrope-sem-build/work-view cargo test --locked --test TARGET TEST_NAME -- --exact`.

| TARGET / TEST_NAME | Meaningful RED | GREEN |
| --- | --- | --- |
| fleet_ui / opening_a_goal_shows_recorded_plan_work_status_owner_and_source_time | no Work view/statuses | passed |
| fleet_ui / explicit_work_is_default_and_explains_its_context_dependencies_and_outcomes | only opaque native fallback rendered | passed |
| fleet_ui / selected_work_keeps_its_identity_and_open_feedback_keeps_its_recipient | w did not select its child owner | passed; peer received child guide/queue/stop after reassignment |
| fleet_ui / visible_work_and_agents_actions_switch_inspection_without_losing_the_goal | missing Work/Agents actions | passed through mouse events |
| fleet_ui / narrow_work_inspection_keeps_current_context_and_control_availability_visible | availability below the visible 60x28 region | passed |
| fleet_ui / commands_inspection_uses_the_selected_work_owner_and_dates_public_narration | missing Commands route | passed; also checks undated update time reset |
| fleet_ui / work_focused_coordinator_and_tracking_actions_stay_explicit_and_request_missing_criteria | no set_goal/final-check request in the no-contract draft | passed; C/T owner routing and saved child draft retained |
| fleet_runtime / clicking_replay_opens_recording_and_preserves_drafts_without_live_actions | no clickable Replay in real PTY | passed; two replay/return cycles and protocol peer saw no live actions |
| console / search_finds_collapsed_work_and_recorded_plan_text_without_changing_selection | work/plan text did not match | passed |
| fleet_ui / unindexed_work_owners_keep_a_useful_recipient_id_and_full_identity_in_details | generic Agent recipient hid identity | passed |
| fleet_ui / native_plan_revisions_do_not_retarget_selected_work_and_coordinator_is_explicit_recovery | expired plan had no explicit Coordinator recovery | passed |

The additional scope/dependency regression, `work_scope_links_go_stale_and_unknown_dependencies_stay_distinct_from_none`, passed without another production change: old scope links show stale, null/empty dependencies differ, and reported completion leaves goal verification unchanged.

## Verification

`cargo fmt --all` and `git diff --check` passed.

`CARGO_TARGET_DIR=/tmp/zoetrope-sem-build/work-view cargo test --locked --quiet --test console --test fleet_ui --test fleet_runtime --test codex_control --test workstate` passed **95 tests**, with the existing manual UI resource probe ignored: console 11, fleet UI 42, PTY lifecycle 6, control 13, state 23.

`CARGO_TARGET_DIR=/tmp/zoetrope-sem-build/work-view-188 cargo +1.88.0 clippy --locked --all-targets --all-features -- -D warnings` passed with no warnings. Root performs final combined native/portable/browser/resource checks and independent review.
