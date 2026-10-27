# SEM-3: Work-first goal inspection

Parent: [semantic context spec](../SEMANTIC-CONTEXT-SPEC.md), [issue #9](https://github.com/wagar-meilech/zoetrope/issues/9). Ticket: [#12](https://github.com/wagar-meilech/zoetrope/issues/12).

Status: implemented, verified and published; closes through PR #8 when merged.

Blocked by: SEM-1 and SEM-2.

## What to build

Goal inspection shows imported native plan statuses/source times plus explicit work items, context, actual owner, declared/unknown dependencies, unresolved IDs, stale outcome links, and verified progress. Explicit reports override native fallback per producer. Selection is stable by explicit work ID and directs steering to actual owner. Label Agents/Commands/Replay routes, preserve narrow terminals/replay/legacy behavior, search work where feasible. Use bounded summary/cached background IO; do not scan whole history per frame. Relevant files: src/console.rs, src/fleet/*, src/provider/work.rs, src/provider/activity.rs, src/provider/metadata.rs, tests/console.rs and tests/fleet_ui.rs. Add metadata fields compatibly.

## Acceptance and process

Follow docs/SEMANTIC-CONTEXT-SPEC.md and the completed questionnaire. Public console and executable/external seams are already approved. Apply the TDD skill: one red test, one green implementation, then the next behavior. Record meaningful RED/GREEN commands and results in docs/research/sem-N-implementation.md (use your ticket number). Keep Rust, no extra model, preserve native history and controls. Implement in your isolated worktree based on feat/goal-console; merge its current tip before reporting complete. Commit with Conventional Commits. Publication and the PR update were explicitly approved by the user on 27 October 2026. Do not merge into main without a user request.

## Local result

See [implementation and validation evidence](../research/sem-3-implementation.md). Published; issue closure occurs through PR #8 when merged.
