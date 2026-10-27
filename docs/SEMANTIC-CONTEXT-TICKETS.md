# Semantic context task graph

Integration: `feat/goal-console`, PR #8. Implements [the spec](SEMANTIC-CONTEXT-SPEC.md) at the approved public console and executable seams. Each ticket uses its own worktree, vertical TDD and red/green evidence.

| Ticket | Blocked by | Complete behavior |
| --- | --- | --- |
| SEM-1: Durable work reports and compact context | None | Agents publish revision-checked work through the executable, restart, inspect compact context and preserve evidence-based progress. |
| SEM-2: Discoverable steering and tracking | None | Operators see recipient-specific capability/actions before sending and prepare coordinator tracking instructions. |
| SEM-3: Work-first goal inspection | SEM-1, SEM-2 | Goal inspection exposes native/explicit work, owners, context, dependencies, source, scope links, progress and owner-directed steering. |
| SEM-4: Reporting convention and final verification | SEM-3 | Reusable project reporting guidance, documented navigation, final checks and resource evidence match combined behavior. |

SEM-1 and SEM-2 proceed independently; SEM-3 consumes their integrated interfaces; SEM-4 completes the feature.

## SEM-1 — Durable work reports and compact context

- Bounded IDs, labels, real owners, statuses and optional purpose/change/next/blocker, parent, dependency and outcome-revision links.
- Restart persistence, stale-edit rejection, identity validation and harmless identical repeats.
- Work completion stays separate from verified evidence.
- Compact context and acknowledgments exclude feedback/question/outbox history.
- Test new behavior through the executable; retain old records/operations.

## SEM-2 — Discoverable steering and tracking

- Recipient and separate guidance/queue/stop availability before sending, including disconnected/unloaded/unsupported states.
- Labeled steering/navigation actions and keyboard/mouse routes; preserve literal text, drafts and deliberate sends.
- Editable coordinator Enable tracking instruction with exact state location and transition-only commands.
- Protocol checks preserve targeting, stale-turn rejection, no rerouting and delivery states.

## SEM-3 — Work-first goal inspection

- Work and verified progress become default selected-goal content.
- Imported plan statuses/source time and explicit work taking precedence per producer.
- Stable explicit selection and actual owner-directed steering.
- Declared/unknown dependencies, unresolved references, context and stale criterion links.
- Labeled Agents, Commands, Replay; preserve legacy navigation, narrow terminals and replay isolation.
- Bounded summaries/background IO and work/plan search where feasible.

## SEM-4 — Reporting convention and verification

- Concise opt-in project guidance and conditional snippet with exact implemented CLI examples.
- Usage/handoff/architecture/validation reflect final behavior.
- Combined native/portable/browser checks, terminal flows and public resource observations.
- Independent standards/spec review and fixes, PR #8 closes new issues; no merge into main.

## Publication status

Automatic approval review rejected publishing the full specification to GitHub without explicit payload/destination approval. GitHub remains the configured tracker. [SEM-1](semantic-tickets/SEM-1.md), [SEM-2](semantic-tickets/SEM-2.md), [SEM-3](semantic-tickets/SEM-3.md) and [SEM-4](semantic-tickets/SEM-4.md) are local task copies pending publication. Implementation proceeds locally; no remote issue creation, push or PR update is claimed.
