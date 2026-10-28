# SEM-1 — Durable work reports and compact context

Implemented on `feat/semantic-work-state`, isolated at `/tmp/zoetrope-sem-state`, based on integration `dd0026f`. Uses the user-approved executable seam; existing public Store interfaces remain available for frontends. No remote publication or native runtime actions.

## Public contract

`WorkRecord.work_items` is a backwards-compatible defaulted `Vec<WorkItem>`. `WorkItem` stores `report: WorkItemInput`, `revision: u64`, and `updated_at: String` (RFC3339). The report is flattened in JSON; Rust callers use `work.report.id`, `work.report.owner`, etc.

`WorkItemInput` contains `id`, `label`, `owner: AgentKey`, `status: WorkStatus`, `source`, optional `parent_id`, `depends_on: Option<Vec<String>>`, `outcome_links: Vec<WorkOutcomeLink>`, and optional `purpose`, `change`, `next`, `blocker`. `WorkOutcomeLink` contains `criterion_id` and `goal_revision`. Status variants are `Planned`, `Active`, `Blocked`, `Done`, `Cancelled`; JSON uses lowercase snake case. Done remains an attributed report, independent of goal verification.

The update is `upsert_work_item`, with required `expected_revision` and `item: WorkItemPatch`. Creation requires expected revision 0 plus id/label/owner/status/source. Later reports require only id and changed fields. Omitted fields retain existing values; explicit null clears parent/dependencies/purpose/change/next/blocker, and [] declares no dependencies. An absent/null dependency value means not declared. outcome_links can be cleared with []. Owner must belong to the same provider-qualified session; existence of a live native owner is not claimed by state storage.

The OS lock covers materialization, revision checks and atomic replacement. Identical reports with current or immediately preceding expected revision return unchanged work; other stale/future reports fail. Revisions advance only for changed work. Failed validation leaves the saved record unchanged. IDs/owner IDs are limited to 128 bytes, labels/source to 512, each context field to 1024, dependencies/outcome links to 64, work items to 512, with the existing 8 MiB record limit preserved. Missing referenced work/criteria are retained as explicit unresolved links for the UI; goal revision zero and empty IDs are invalid.

Creation:

```bash
zoe state --root STATE apply codex SESSION '{"op":"upsert_work_item","expected_revision":0,"item":{"id":"steering","label":"Expose available input","owner":{"session":{"provider":"codex","session_id":"SESSION"},"agent_id":"main"},"status":"active","source":"coordinator","purpose":"Explain input before sending"}}'
```

Transition:

```bash
zoe state --root STATE apply codex SESSION '{"op":"upsert_work_item","expected_revision":1,"item":{"id":"steering","status":"done","blocker":null}}'
```

Work writes acknowledge only `{saved,session,goal_revision,work_item:{id,revision,updated_at}}`; use the returned revision for the next transition. They do not echo report text or saved history.

## Compact context

```bash
zoe state --root STATE context codex SESSION
zoe state --root STATE context codex SESSION --offset 16 --limit 8
```

Default work page: 16, maximum requested limit: 32. The total output including newline is at most 64 KiB. Projection includes session, goal title/revision and compact criterion id/label/status/weight/evidence_count, progress, work items and recorded native plans. Drafts/questions/outbox/scope history/evidence bodies are excluded. Complete stored reports are unchanged.

The response also contains `work_offset`, `work_items_total`, `work_items_omitted`, `criteria_omitted`, `plans_omitted`, `plan_steps_omitted`, `next_work_offset`, `context_max_bytes`. Text is clipped at 256 characters with explicit truncation flags; work references are limited to 16 with per-item omitted counts. Native plan context includes at most four plans/eight steps each; criterion context includes at most 32 entries, subject to byte budgets. `next_work_offset` supports continuing work pages. Full `show` remains an explicit operation when complete stored details are needed.

## Vertical TDD evidence

For each row, ran `CARGO_TARGET_DIR=/tmp/zoetrope-sem-build/state cargo test --locked --test workstate TEST_NAME -- --exact` first RED and then GREEN after the smallest implementation change.

| TEST_NAME | Observed RED | GREEN |
| --- | --- | --- |
| work_reports_survive_restart_without_verifying_goal_outcomes | unknown upsert_work_item variant | 1 passed |
| concurrent_work_revisions_reject_stale_reports_and_accept_identical_retries | retry duplicated saved work | 1 passed |
| work_context_preserves_declared_dependencies_and_revision_qualified_outcomes | parent_id was missing | 1 passed |
| transition_reports_update_only_supplied_fields_and_can_clear_context_explicitly | missing label on transition | 1 passed |
| invalid_work_reports_cannot_change_an_existing_record | cross-session owner was accepted | 1 passed |
| compact_context_reads_current_work_and_plans_without_echoing_private_history | unknown context command | 1 passed |
| context_pages_work_and_bounds_large_goal_and_plan_text | output was 346,485 bytes | 1 passed |
| work_receipts_return_the_saved_revision_without_echoing_report_context | missing receipt work ID/revision | 1 passed |
| an_identical_report_with_an_unrelated_revision_is_not_acknowledged_as_a_retry | expected revision 99 was accepted | 1 passed |

Final focused suite: `CARGO_TARGET_DIR=/tmp/zoetrope-sem-build/state cargo test --locked --test workstate` — 23 passed. `cargo fmt --check` and `git diff --check` passed.

Default Rust 1.99 strict all-target/all-feature clippy reported 29 existing collapsible-if/match lints in baseline provider/control/fleet and the pre-existing question-resolution code. None point at added work-report code. Root independently confirmed this baseline and handles final combined toolchain validation. `CARGO_TARGET_DIR=/tmp/zoetrope-sem-build/state-188 cargo +1.88.0 clippy --locked --all-targets --all-features -- -D warnings` passed with no warnings on the documented native toolchain.
