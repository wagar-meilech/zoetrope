# SEM-2: Discoverable steering and tracking

Parent: [semantic context spec](../SEMANTIC-CONTEXT-SPEC.md), [issue #9](https://github.com/wagar-meilech/zoetrope/issues/9). Ticket: [#11](https://github.com/wagar-meilech/zoetrope/issues/11).

Status: implemented, verified and published; closes through PR #8 when merged.

Blocked by: None.

## What to build

Make selected-recipient guidance/queue/stop availability visible before sending. Explain disconnected/unloaded and false direct-input capability without conflating interruption. Add discoverable labeled controls, deliberate non-F-key alternatives and mouse actions; editors remain literal. Enable tracking prepares an editable instruction for the explicit session coordinator, exact state root/session, compact `zoe state context` reads and transition-only `upsert_work_item` reports (SEM-1 adds these command names). A coordinator-selection action must be explicit. Cover external behavior through tests/fleet_ui.rs and local protocol peer. Limit code to src/fleet/* and tests/fleet_ui.rs (test helpers may be extended). Avoid work-item UI, which SEM-3 adds after integration. Preserve old input behaviors where they remain compatible.

## Acceptance and process

Follow docs/SEMANTIC-CONTEXT-SPEC.md and the completed questionnaire. Public console and executable/external seams are already approved. Apply the TDD skill: one red test, one green implementation, then the next behavior. Record meaningful RED/GREEN commands and results in docs/research/sem-N-implementation.md (use your ticket number). Keep Rust, no extra model, preserve native history and controls. Implement in your isolated worktree based on feat/goal-console; merge its current tip before reporting complete. Commit with Conventional Commits. Publication and the PR update were explicitly approved by the user on 27 October 2026. Do not merge into main without a user request.

## Local result

See [implementation and validation evidence](../research/sem-2-implementation.md). Published; issue closure occurs through PR #8 when merged.
