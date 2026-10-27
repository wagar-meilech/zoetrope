# SEM-1: Durable work reports and compact context

Parent: [semantic context spec](../SEMANTIC-CONTEXT-SPEC.md).

Status: implemented and integrated locally in `1e5fba5`; remote publication pending approval.

Blocked by: None.

## What to build

Bounded explicit work reports through the executable: stable IDs, real AgentKey owner, planned/active/blocked/done/cancelled statuses, source/update time/revision, optional parent/dependencies, goal-revision-qualified outcome links, purpose/change/next/blocker. Reject stale updates and cross-session owners atomically, preserve restart/old-record compatibility, and make identical repeat reports harmless. Add compact context read excluding drafts/questions/outbox and keep apply receipts small. Reported completion never verifies outcomes. Work only in src/workstate.rs (and new subordinate modules if helpful), tests/workstate.rs and your implementation notes. Coordinate the canonical public fields via notes; UI consumers follow later.

## Acceptance and process

Follow docs/SEMANTIC-CONTEXT-SPEC.md and the completed questionnaire. Public console and executable/external seams are already approved. Apply the TDD skill: one red test, one green implementation, then the next behavior. Record meaningful RED/GREEN commands and results in docs/research/sem-N-implementation.md (use your ticket number). Keep Rust, no extra model, preserve native history and controls. Implement in your isolated worktree based on feat/goal-console; merge its current tip before reporting complete. Commit with Conventional Commits. Do not push or publish remotely.

## Local result

See [implementation and validation evidence](../research/sem-1-implementation.md). Publication remains pending; no remote issue closure is claimed.
