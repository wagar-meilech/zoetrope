# WG-1: Navigate a real work/dependency graph

## Parent

[Approved work graph specification](../WORK-GRAPH-SPEC.md), [issue #15](https://github.com/wagar-meilech/zoetrope/issues/15). This ticket is [#16](https://github.com/wagar-meilech/zoetrope/issues/16).

## What to build

Deliver the approved vertical behavior described by this ticket, following the specification and answered questionnaire. Preserve Rust/local efficiency, source attribution, deliberate exact-recipient control and portable replay.

## Acceptance criteria

- [x] Existing explicit reports and native plans appear as selectable task nodes; only declared prerequisites become dependency edges.
- [x] Fleet and Work are distinct views: w/Enter opens a goal, breadcrumb/back restores Fleet camera and selection, and Agents/Replay remain explicit.
- [x] Stable task selection, owner-directed inspection, useful details, disconnected nodes, missing references, parent grouping and cycles remain usable under bounded layout.
- [x] Public keyboard/mouse/render tests establish the complete flow without testing private layout helpers.

## Blocked by

None.

## Implementation process

Use an isolated worktree from the integration branch, merge its current tip before handoff, and apply vertical TDD at the already approved public console and executable/external seams. Record RED/GREEN and checks in a ticket implementation note. Commit with Conventional Commits; merger integration and final independent review follow. Do not change another live project or merge into main.

## Implementation evidence

See [ticket implementation](../research/wg-1-implementation.md) and the
[final combined validation](../WORK-GRAPH-VALIDATION.md).
