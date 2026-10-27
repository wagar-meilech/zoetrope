# WG-4: Activate the reusable tracking skill

## Parent

[Approved work graph specification](../WORK-GRAPH-SPEC.md).

## What to build

Deliver the approved vertical behavior described by this ticket, following the specification and answered questionnaire. Preserve Rust/local efficiency, source attribution, deliberate exact-recipient control and portable replay.

## Acceptance criteria

- [ ] A packaged normally discoverable tracking skill integrates through the selected project tracker convention without rewriting upstream skills.
- [ ] One explicit setup adds only managed project opt-in/configuration and conditional pointers; later goals bind exact native identity and maintain ordinary artifacts.
- [ ] The skill recognizes existing structured documents and supports a minimal non-Matt fallback without maintaining a second adequate plan.
- [ ] Enable tracking prepares an editable short exact-target request naming the installed skill/executable, while fallback reporting stays available.
- [ ] Installation/update ships the validated skill; disposable-project behavioral exercises cover setup, resume and fallback.

## Blocked by

WG-3.

## Implementation process

Use an isolated worktree from the integration branch, merge its current tip before handoff, and apply vertical TDD at the already approved public console and executable/external seams. Record RED/GREEN and checks in a ticket implementation note. Commit with Conventional Commits; merger integration and final independent review follow. Do not change another live project or merge into main.
