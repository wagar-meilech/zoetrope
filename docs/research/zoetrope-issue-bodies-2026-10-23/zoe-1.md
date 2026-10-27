Create the public console interface and stable identities above the existing single-session viewer. Reuse discovery, include archived history where available, classify internal helpers correctly and index metadata incrementally. Establish selection, expansion, current/history filtering and search through externally observable behavior. Preserve legacy session parsing and replay.

Acceptance: fixture sessions from different projects and providers are discoverable without selecting a directory; archived work appears in history; children remain attached to the correct root; equal agent identifiers in different sessions cannot collide; malformed files do not break discovery; reindexing does not move selection unexpectedly.

## Context

Spec: https://github.com/wagar-meilech/zoetrope/issues/1

Integration branch: `feat/goal-console`.

## Dependencies

- No blocking tickets.

## Implementation workflow

Use the approved public console and executable/external integration seams. Follow vertical TDD: observe red, implement one behavior, observe green. Work in an isolated worktree based on the latest integration branch and merge that tip before handoff. Do not alter unrelated live Codex sessions.
