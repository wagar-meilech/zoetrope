# SEM-1: Durable work reports and compact context

Parent: [semantic context spec](../SEMANTIC-CONTEXT-SPEC.md), [issue #9](https://github.com/wagar-meilech/zoetrope/issues/9). Ticket: [#10](https://github.com/wagar-meilech/zoetrope/issues/10).

Status: implemented, verified and published; closes through PR #8 when merged.

Blocked by: None.

## What to build

Bounded explicit work reports through the executable: stable IDs, real AgentKey owner, planned/active/blocked/done/cancelled statuses, source/update time/revision, optional parent/dependencies, goal-revision-qualified outcome links, purpose/change/next/blocker. Reject stale updates and cross-session owners atomically, preserve restart/old-record compatibility, and make identical repeat reports harmless. Add compact context read excluding drafts/questions/outbox and keep apply receipts small. Reported completion never verifies outcomes. Work only in src/workstate.rs (and new subordinate modules if helpful), tests/workstate.rs and your implementation notes. Coordinate the canonical public fields via notes; UI consumers follow later.

## Acceptance and process

Follow docs/SEMANTIC-CONTEXT-SPEC.md and the completed questionnaire. Public console and executable/external seams are already approved. Apply the TDD skill: one red test, one green implementation, then the next behavior. Record meaningful RED/GREEN commands and results in docs/research/sem-N-implementation.md (use your ticket number). Keep Rust, no extra model, preserve native history and controls. Implement in your isolated worktree based on feat/goal-console; merge its current tip before reporting complete. Commit with Conventional Commits. Publication and the PR update were explicitly approved by the user on 27 October 2026. Do not merge into main without a user request.

## Local result

See [implementation and validation evidence](../research/sem-1-implementation.md). Published; issue closure occurs through PR #8 when merged.
