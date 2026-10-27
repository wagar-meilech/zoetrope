Exercise the combined workflows, document commands and limitations, update the old read-only contract for explicit local controls, and run repository checks. Install the rebuilt fork at the existing `zoe` command, validate a real terminal flow and MD Reader preview, resolve final code-review findings, then prepare the integration PR for review.

Acceptance: agreed behavior passes at both public seams; existing replay tests remain green; native release and portable core build; relevant browser checks pass; no new warnings; user-facing docs explain current/history, search, progress, steering, persistent questions, documents and compatibility limits; final report distinguishes verified behavior from unsupported cases.

## Context

Spec: https://github.com/wagar-meilech/zoetrope/issues/1

Integration branch: `feat/goal-console`.

## Dependencies

- Blocked by https://github.com/wagar-meilech/zoetrope/issues/5
- Blocked by https://github.com/wagar-meilech/zoetrope/issues/6

## Implementation workflow

Use the approved public console and executable/external integration seams. Follow vertical TDD: observe red, implement one behavior, observe green. Work in an isolated worktree based on the latest integration branch and merge that tip before handoff. Do not alter unrelated live Codex sessions.
