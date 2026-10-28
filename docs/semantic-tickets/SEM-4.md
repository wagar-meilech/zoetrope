# SEM-4: Reporting convention and final verification

Parent: [semantic context spec](../SEMANTIC-CONTEXT-SPEC.md), [issue #9](https://github.com/wagar-meilech/zoetrope/issues/9). Ticket: [#13](https://github.com/wagar-meilech/zoetrope/issues/13).

Status: implemented, verified and published; closes through PR #8 when merged.

Blocked by: SEM-3.

## What to build

Write concise opt-in project reporting guidance and a small conditional instruction snippet with actual CLI examples; no automatic edits to user project/global instructions. Update usage/handoff/architecture and validation with implemented behavior, limits, final test/resource evidence. Main work includes public tests/fixtures, full checks, independent review fixes and manual terminal/resource verification. The user explicitly approved publication to this GitHub repository and updating PR #8 on 27 October 2026. Preserve dependency links and close through that PR.

## Acceptance and process

Follow docs/SEMANTIC-CONTEXT-SPEC.md and the completed questionnaire. Public console and executable/external seams are already approved. Apply the TDD skill: one red test, one green implementation, then the next behavior. Record meaningful RED/GREEN commands and results in docs/research/sem-N-implementation.md (use your ticket number). Keep Rust, no extra model, preserve native history and controls. Implement in your isolated worktree based on feat/goal-console; merge its current tip before reporting complete. Commit with Conventional Commits. Publication and the PR update were explicitly approved by the user on 27 October 2026. Do not merge into main without a user request.

## Local result

See [documentation and example verification](../research/sem-4-implementation.md). Final combined tests, resources and both review axes are complete: [validation](../SEMANTIC-CONTEXT-VALIDATION.md).
