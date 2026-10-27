# SEM-4: Reporting convention and final verification

Parent: [semantic context spec](../SEMANTIC-CONTEXT-SPEC.md).

Status: reporting/documentation ready locally; final combined validation and independent review pending. Remote publication remains pending approval.

Blocked by: SEM-3.

## What to build

Write concise opt-in project reporting guidance and a small conditional instruction snippet with actual CLI examples; no automatic edits to user project/global instructions. Update usage/handoff/architecture and validation with implemented behavior, limits, final test/resource evidence. Main work includes public tests/fixtures, full checks, independent review fixes and manual terminal/resource verification. Remote publishing is blocked by auto-review pending explicit payload/destination approval; do not publish/push or modify PR remotely. GitHub remains configured tracker; local ticket copies are pending publication.

## Acceptance and process

Follow docs/SEMANTIC-CONTEXT-SPEC.md and the completed questionnaire. Public console and executable/external seams are already approved. Apply the TDD skill: one red test, one green implementation, then the next behavior. Record meaningful RED/GREEN commands and results in docs/research/sem-N-implementation.md (use your ticket number). Keep Rust, no extra model, preserve native history and controls. Implement in your isolated worktree based on feat/goal-console; merge its current tip before reporting complete. Commit with Conventional Commits. Do not push or publish remotely.

## Local result

See [documentation and example verification](../research/sem-4-implementation.md). Root performs final combined tests/resources/review before marking SEM-4 complete.
