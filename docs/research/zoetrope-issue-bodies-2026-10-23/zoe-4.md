Wire the console into default launch. Add the global graph, one-key history switch, search, lazy expansion and pinned selection. Surface goal progress, concise activities, persistent feedback and questions, document selection and clear controls. Keep original single-session detail and playback available.

Acceptance: public input/render tests demonstrate a user finding a prior goal, selecting its agent, drafting feedback, navigating away and back without losing it, giving guidance to the intended live target, and reading a goal's criteria/evidence. Narrow terminals show usable content and key help; normal typing never triggers graph shortcuts accidentally.

## Context

Spec: https://github.com/wagar-meilech/zoetrope/issues/1

Integration branch: `feat/goal-console`.

## Dependencies

- Blocked by https://github.com/wagar-meilech/zoetrope/issues/3
- Blocked by https://github.com/wagar-meilech/zoetrope/issues/4

## Implementation workflow

Use the approved public console and executable/external integration seams. Follow vertical TDD: observe red, implement one behavior, observe green. Work in an isolated worktree based on the latest integration branch and merge that tip before handoff. Do not alter unrelated live Codex sessions.
