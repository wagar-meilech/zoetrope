# SEM-2 — Discoverable steering and tracking

Implemented on `feat/semantic-steering`, based on `feat/goal-console` at `dd0026f`, in `/tmp/zoetrope-sem-steering`. Remote publication/push remains pending explicit payload/destination approval; this ticket performs no remote mutation.

## Behavior

- Cached per-action preflight shows Guidance, Queue and Stop availability before sending. Direct-input restriction blocks guidance/queue while exact active-turn interruption remains available. Missing connection, unloaded thread, unavailable state and missing active turn have explicit reasons. Runtime validation still checks the actual target/turn at delivery.
- Selected work exposes clickable **Steer**, **Coordinator** and **Enable tracking** actions. Feedback has clickable Send/Queue/Stop and deliberate `Ctrl-S`/`Ctrl-Q`/`Ctrl-X` alternatives to F2/F3/F4. Ordinary editor characters retain their text meaning; send-key repeats are ignored.
- Selecting Coordinator is explicit and preserves saved child drafts. Tracking setup targets the selected session's coordinator, appends an editable instruction to existing coordinator feedback, preserves child feedback, and saves without sending. The instruction uses exact state-root/provider/session, `zoe state context`, `upsert_work_item`, `--help` and reports at meaningful transitions.
- `control_target()` and `control_availability(mode)` centralize recipient/preflight behavior. SEM-3 can resolve a selected work owner through `selected_agent()`; the rest of the send/capability paths follow it.
- Rendering/mouse routing use cached state and bounded action hit regions; no filesystem/socket work or model call was added to drawing.

## TDD evidence

All tests use the approved public console input/render and executable/external protocol seams. Existing state interfaces prepare fixtures; persisted output is inspected with the actual executable. Build artifacts are isolated with `CARGO_TARGET_DIR=/tmp/zoetrope-sem-build/steering`.

Each command below was run RED before its implementation and then GREEN:

1. `cargo test --locked --test fleet_ui restricted_child_explains_input_before_sending_and_keeps_stop_available -- --exact` — RED: absent preflight text; GREEN: restrictions shown before sends and peer receives only the selected child's stop.
2. `cargo test --locked --test fleet_ui visible_steering_actions_and_control_shortcuts_keep_text_literal -- --exact` — RED: absent Steer button; GREEN: real mouse and Ctrl actions reach the protocol peer, literal shortcut letters remain text. The new mouse test was corrected to use display columns rather than UTF-8 byte offsets for box-drawing glyphs.
3. `cargo test --locked --test fleet_ui tracking_is_an_editable_coordinator_draft_and_keeps_existing_feedback -- --exact` — RED: no tracking action; GREEN: coordinator draft has compact commands and preserves prior recipient drafts, with no outbox delivery. The assertion uses stable instruction content rather than a transient save status.
4. `cargo test --locked --test fleet_ui explicitly_selecting_coordinator_keeps_the_child_draft_with_its_owner -- --exact` — RED: coordinator action absent; GREEN: child/coordinator drafts remain separately addressed after explicit selection.

Combined verification: `cargo fmt --all`; `cargo test --locked --test fleet_ui --test codex_control` — 45 passed, one existing manual resource probe ignored.

This uses an independent local protocol peer, not a model session or internal collaborator mock. It establishes UI/control behavior, not universal installed-runtime compatibility or agent reporting compliance. No actual native session was steered.

## Final ticket verification

- `cargo test --locked --test fleet_runtime`: 5 passed after correcting the independent PTY adapter. Its old concatenation of emitted words skipped an unchanged `e` after the layout changed; the actual saved draft was independently confirmed through `zoe state show`. Compact PTY assertions now reconstruct visible cells from cursor-positioned output with Python stdlib. This is test-harness handling, not a persistence behavior change.
- Formatting and `git diff --check` passed; Python support helpers compile.
- Strict `cargo clippy --locked --all-targets --all-features -- -D warnings` on this environment's only installed Rust/Clippy 1.99 initially reported 30 style diagnostics. The one introduced in this ticket's new mouse routing was corrected. The other 29 are existing `collapsible_if` / `collapsible_match` findings across untouched provider/control/state and existing fleet logic, left for final integration review. The project declares Rust 1.88; no older toolchain is installed here.

Final combined command: `cargo test --locked --test fleet_ui --test codex_control --test fleet_runtime` — 50 passed, one existing ignored manual probe. `cargo clippy --locked --all-targets --all-features -- -D warnings -A clippy::collapsible_if -A clippy::collapsible_match` passed, isolating the documented baseline style categories; this is not a claim that unrestricted strict clippy passed.
