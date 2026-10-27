# SEM-3: Work-first goal inspection

Parent: [semantic context spec](../SEMANTIC-CONTEXT-SPEC.md).

Status: implemented and integrated locally in `e6a6d52 (detail correction ec35ad3)`; remote publication pending approval.

Blocked by: SEM-1 and SEM-2.

## What to build

Goal inspection shows imported native plan statuses/source times plus explicit work items, context, actual owner, declared/unknown dependencies, unresolved IDs, stale outcome links, and verified progress. Explicit reports override native fallback per producer. Selection is stable by explicit work ID and directs steering to actual owner. Label Agents/Commands/Replay routes, preserve narrow terminals/replay/legacy behavior, search work where feasible. Use bounded summary/cached background IO; do not scan whole history per frame. Relevant files: src/console.rs, src/fleet/*, src/provider/work.rs, src/provider/activity.rs, src/provider/metadata.rs, tests/console.rs and tests/fleet_ui.rs. Add metadata fields compatibly.

## Acceptance and process

Follow docs/SEMANTIC-CONTEXT-SPEC.md and the completed questionnaire. Public console and executable/external seams are already approved. Apply the TDD skill: one red test, one green implementation, then the next behavior. Record meaningful RED/GREEN commands and results in docs/research/sem-N-implementation.md (use your ticket number). Keep Rust, no extra model, preserve native history and controls. Implement in your isolated worktree based on feat/goal-console; merge its current tip before reporting complete. Commit with Conventional Commits. Do not push or publish remotely.

## Local result

See [implementation and validation evidence](../research/sem-3-implementation.md). Publication remains pending; no remote issue closure is claimed.
