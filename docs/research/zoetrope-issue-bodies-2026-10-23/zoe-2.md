Add local goal contracts with evidence and scope revisions; draft/queued feedback; unanswered questions and partial answers; generated document references. Provide public local commands for reading/updating these records and parsing supported transcript updates. Preserve unknown/expired states honestly and never mutate native history.

Acceptance: evidence-backed progress and its count agree with independent examples; missing tracking remains unknown; scope changes preserve prior evidence; process restart restores unfinished answers and feedback; expired questions are retained without false delivery; documents retain exact paths and producer identity.

## Context

Spec: https://github.com/wagar-meilech/zoetrope/issues/1

Integration branch: `feat/goal-console`.

## Dependencies

- Blocked by https://github.com/wagar-meilech/zoetrope/issues/2

## Implementation workflow

Use the approved public console and executable/external integration seams. Follow vertical TDD: observe red, implement one behavior, observe green. Work in an isolated worktree based on the latest integration branch and merge that tip before handoff. Do not alter unrelated live Codex sessions.
