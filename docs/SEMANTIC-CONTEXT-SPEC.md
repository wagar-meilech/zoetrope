# Semantic goal context and discoverable steering

Status: mechanism approved and implementation authorized on 27 October 2026 through the completed questionnaire and “Answered, proceed.” Continues the existing integration and its two approved public testing seams.

## Problem Statement

A goal may occupy one coordinator for a long time while Zoetrope shows agents and technical activity rather than meaningful steps toward the goal. Imported plan statuses are hidden, outcome evidence is disconnected from current work, and steering capability is explained after attempting delivery. The operator needs direction, remaining work, blockers and the real recipient without paying for a second monitoring model.

## Solution

Show meaningful work and evidence-backed outcomes together. Work has explicit identity, actual ownership, attributed status, concise context and optional declared dependencies/outcome links. Existing plans and public commentary provide a passive baseline; working agents enrich it with compact local reports at meaningful transitions. Visible steering/tracking actions use the existing runtime. Reusable opt-in project guidance reduces setup. Extra model assistance is deferred.

## User Stories

1. As an operator, I want meaningful work within a coordinator's goal, so that sequential work is understandable.
2. As an operator, I want recorded plan statuses displayed, so that existing information is useful immediately.
3. As an operator, I want reported purpose, change, next action and blockers, so that I can assess direction.
4. As an operator, I want actual ownership, so that I know whom to steer.
5. As an operator, I want declared dependencies distinguished from unknown information, so that displayed order does not invent blocking.
6. As an operator, I want reported work completion separate from verified outcomes, so that progress stays honest.
7. As an operator, I want scope-qualified outcome links, so that outdated coverage remains visibly stale.
8. As an operator, I want work and progress emphasized when opening goals, so that context is discoverable.
9. As an operator, I want labeled Agents, Commands and Replay routes, so that inspection is understandable.
10. As an operator, I want stable work selection during updates, so that feedback remains attached to intended work.
11. As an operator, I want source/update time, so that old reports are distinguishable from current activity.
12. As an operator, I want guidance/queue/stop availability before sending, so that restrictions are explained early.
13. As an operator, I want work-directed guidance addressed to its actual owner, so that other agents are not redirected.
14. As an operator, I want explicit coordinator selection for unreachable children, so that recipient changes are deliberate.
15. As an operator, I want an editable Enable tracking instruction, so that existing sessions can report work.
16. As an operator, I want reports at meaningful transitions, so that routine file operations add no reporting overhead.
17. As an operator, I want reusable selected-project guidance, so that setup need not be repeated in full.
18. As an agent, I want compact context/acknowledgments, so that reporting does not echo drafts or history.
19. As an agent, I want atomic revision-checked updates, so that concurrent reports cannot silently overwrite newer work.
20. As an operator, I want missing context labeled honestly, so that intent is not invented.
21. As an operator, I want lightweight local monitoring, so that viewing invokes no model.
22. As an operator, I want native access, replay and existing controls preserved, so that this remains an additional interface.

## Implementation Decisions

- Extend existing Rust state, provider projection and fleet console. Preserve incremental reads, bounded caches, lazy details, background IO, idle event-driven rendering and portable replay.
- Separate outcome verification from attributed work reports. Work covers planned, active, blocked, reported done and cancelled; reported completion never verifies criteria.
- Explicit work carries stable publisher-chosen ID, label, actual owner, status, provenance, update time and advancing revision. Optional fields carry parent, declared dependencies, goal-revision-qualified criterion links and purpose/change/next/blocker. Unknown dependencies differ from an explicit empty list. Bound strings/collections.
- Extend the local state command for work reports. Reject stale revisions/invalid identities without mutation, make identical repeated reports harmless, retain identities across restart and preserve old-record compatibility.
- Add compact context reading: current goal/progress and bounded work/plan context, excluding feedback/questions/delivery history. Apply acknowledgments stay short; full inspection stays explicit.
- Render actual recorded native plan statuses as an attributed fallback, with source time when recorded and unknown otherwise. Native plan changes are revisions; title/index similarity does not prove durable identity. Explicit work takes precedence for its producer.
- Show declared/unknown dependencies, unresolved references and stale outcome links. Scope changes preserve earlier evidence. Historical ownership does not establish live capability.
- Goal inspection emphasizes work and verified outcomes. Keep fleet navigation compact, layout/selection stable and narrow terminals usable, with labeled Agents/Commands/Replay. Selected-goal work nodes/list are acceptable; individual commands do not become graph nodes.
- Work selection resolves the real owner. Preserve exact-thread/turn controls, drafts and receipts. Explain separate guidance/queue/stop availability before sending; a direct-input restriction does not imply interruption is unsupported. Coordinator selection is explicit.
- Provide discoverable keyboard and mouse actions, including deliberate alternatives to function-key sends. Typed characters remain literal in editors. Disabled actions explain why.
- Enable tracking prepares an editable coordinator instruction with exact provider/session/state location and transition-only reporting commands. Only deliberate guidance/queue sends it; acceptance is not proof that reporting will occur. Native tracking and criteria requests name the running executable and exact absolute state root, and use filtered compact context rather than the full saved record.
- Provide concise reusable project reporting guidance and opt-in snippet. The operator chooses projects; a small conditional pointer loads details only for tracked work. Do not automatically rewrite project/global instructions.
- Public commentary/explicit records supply semantics. Internal KV caches, opaque compaction and private reasoning are not work-fact sources.
- Preserve native transcripts/policies, ordinary Codex, questions/drafts/documents, legacy commands and browser replay.

## Testing Decisions

Use the existing approved seams:

1. Public console input/render plus external runtime messages: work/status/source/ownership, dependencies, stale links, progress, capability, targeting, drafts, tracking, narrow terminals and replay isolation. Misses agent-report correctness and terminal-specific rendering.
2. Executable and external connections: work updates/context, revision handling, bounded output, restart persistence and guarded local protocol exchange. Misses universal runtime-version compatibility.

Prior art: fixture-based console tests, actual state-command processes and local protocol peers. Test external behavior, not private methods or internal collaborators. Follow vertical TDD, recording red/green commands per complete behavior. Existing public state interfaces can prepare fixtures; new state behavior is verified through the executable.

Run formatting, locked native tests, strict clippy, portable checks and relevant browser compatibility. Verify representative large-history/idle behavior via the public interface without timing-fragile assertions. Live mutations use explicitly disposable sessions only.

Baseline: 290 native tests passed, one manual probe ignored. New tests begin after the mechanism discussion and authorization above.

## Out of Scope

- Extra model clients, summarizers, classifiers or spending configuration this iteration. Future helpers require explicit global/project opt-in and agreed role/provider/data access/trigger/budget.
- Changing scheduling/delegation or replacing the harness.
- Inferring dependencies, intent or verification from routine actions, time, tokens or finished-agent counts.
- Reading KV state, decoding opaque compaction or claiming history equals current model input.
- Automatic instruction/policy edits, guidance sends, recipient rerouting, uncertain-delivery retries or merge into main.

## Further Notes

The completed questionnaire is the product source of truth; the mechanism document records reasoning/cache research. Use the existing feature integration branch/PR, dependency-aware tickets, separate implementer worktrees, merger integration and independent standards/spec review.
