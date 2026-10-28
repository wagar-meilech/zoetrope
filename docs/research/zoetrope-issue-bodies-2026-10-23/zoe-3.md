Implement compatible shared-runtime attachment, live metadata and exact target/turn validation. Separate guidance, native queueing, interruption and answers. Preserve native Codex access and approval policy. Verify coexistence using disposable sessions; degrade unsupported actions explicitly without duplicate runs or hidden parent routing.

Acceptance: external protocol-peer tests prove exact recipient and active-turn checks, failure/disconnection behavior, delivery-state distinctions, and replay isolation. A real local compatibility check establishes which actions can coexist with native Codex. No passive monitoring path invokes a model.

## Context

Spec: https://github.com/wagar-meilech/zoetrope/issues/1

Integration branch: `feat/goal-console`.

## Dependencies

- Blocked by https://github.com/wagar-meilech/zoetrope/issues/2

## Implementation workflow

Use the approved public console and executable/external integration seams. Follow vertical TDD: observe red, implement one behavior, observe green. Work in an isolated worktree based on the latest integration branch and merge that tip before handoff. Do not alter unrelated live Codex sessions.
