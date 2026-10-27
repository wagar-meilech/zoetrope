# Semantic-context review fixes — 27 October 2026

Local branch `feat/semantic-review-fixes`, based on `ec35ad3`, with the current
integration documentation at `cd41785` merged before final validation. The final
checked commit identity is supplied in the integration handoff.

## Confirmed findings and behavior

- Empty native `update_plan` revisions now replace the previous proposal for the
  same producer. Withdrawn steps disappear, their selected revision expires, and
  later native work cannot silently become that selection or steering recipient.
- Relative fleet state roots resolve once against the UI working directory before
  inventory, state and control workers start. Tracking (`T`) and criteria (`A`)
  commands share that exact absolute root when the agent runs in another project.
  A failed working-directory lookup is displayed; command generation refuses an
  unresolved relative root. Rendering performs no filesystem lookup.
- Native plan status interpretation now belongs to one provider-owned function.
  Shared `PlanStatus` variants feed semantic selection and presentation separately.
  Its legacy saved-string decoder delegates to that same provider function.
  Stored/context JSON keeps `pending`, `in_progress` and `completed`; the older
  `inProgress` spelling remains accepted. Unknown strings and absent status stay
  attributed/unknown rather than gaining completion or verification semantics.
  Existing string `.into()` fixture constructors remain usable.
- Compact details, full details and work focus share one current-entry helper:
  explicit selection, otherwise active, blocked, then first work. An expired
  explicit selection returns no current entry until the operator chooses work.
  Display wording no longer decides selection priority.
- Rust 1.99 Clippy's machine-applicable collapsible-condition/match suggestions
  were applied and inspected in seven existing source files. The tool reported
  30 applications covering the reported baseline diagnostics. Conditions retain
  their order and short-circuit behavior; there are no lint suppressions.

## Vertical TDD evidence

Both regressions use the already approved public FleetConsole input/render and
executable state interfaces. Fixtures are disposable local transcripts, state
stores and unavailable runtime sockets. No real native session or model was used.

Empty-plan slice:

```sh
CARGO_TARGET_DIR=/tmp/zoetrope-sem-build/review-fixes cargo test --locked --test fleet_ui withdrawing_a_native_plan_clears_work_and_expires_its_selected_entry -- --exact
```

RED: failed waiting for `No reported work`; the selected withdrawn step and its
old proposal stayed visible. GREEN: passed after removing the importer’s empty
plan early return. The same test appends later replacement work and verifies that
the old selection stays unavailable rather than rerouting.

Relative-root slice:

```sh
CARGO_TARGET_DIR=/tmp/zoetrope-sem-build/review-fixes cargo test --locked --test fleet_ui tracking_and_criteria_commands_share_the_ui_store_across_project_directories -- --exact
```

RED: generated tracking command retained a relative `--root`. GREEN: passed after
one-time root resolution. The public test persists both `T` and `A` drafts, reads
that same record using the actual executable from a different agent project,
and checks the read/apply commands contain the UI store’s exact absolute path.
It does not change process working directory or global environment.

The review-stage status/selection refactor reused existing public regressions;
existing render/context fixtures were extended with the legacy alias, an unknown
status and an absent status. No private-method or collaborator-mock tests added.

## Validation

After all source/test edits, native validation passed:

```sh
CARGO_TARGET_DIR=/tmp/zoetrope-sem-build/review-fixes cargo test --locked --all-features
CARGO_TARGET_DIR=/tmp/zoetrope-sem-build/review-fixes cargo clippy --locked --all-targets --all-features -- -D warnings
CARGO_TARGET_DIR=/tmp/zoetrope-sem-build/review-fixes-msrv cargo +1.88.0 clippy --locked --all-targets --all-features -- -D warnings
cargo fmt --all --check
git diff --check
```

317 native tests passed: 207 library, 8 executable, 13 control, 11 console,
5 document launch, 6 fleet lifecycle, 44 fleet UI and 23 workstate. The existing
manual fleet resource probe stays ignored. Strict Clippy passed on both Rust
1.99 and the declared Rust 1.88 minimum. The exact committed tree is rechecked
before handoff; integration owns the final browser/resource/package evidence.

No dependency, hidden model, remote communication/publication, permission-policy
change or native-transcript write was introduced. All work remains local.
