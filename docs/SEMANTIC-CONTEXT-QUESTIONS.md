# Semantic context, steering and progress — design questions

Status: Rounds 1 and 2 answered; implementation authorized on 27 October 2026 by “Answered, proceed.” This questionnaire continues the existing goal-console work; it does not replace the earlier approved decisions.

The agreed mechanism is in [SEMANTIC-CONTEXT-DESIGN.md](SEMANTIC-CONTEXT-DESIGN.md), with the implementation contract in [SEMANTIC-CONTEXT-SPEC.md](SEMANTIC-CONTEXT-SPEC.md). Your Round 2 answers complete the mechanism discussion requested in Q1. The optional model remains deferred; reusable project reporting guidance is included alongside per-goal tracking.

Check your preferred option, write in the answer areas, or edit the questions directly. A written answer takes precedence over a checkbox. Recommendations are not selected automatically. You can answer some questions now and return to the others later.

## Already established

- Keep the main app in Rust, lightweight and efficient.
- Keep model-token usage as low as practical.
- Make meaningful work understandable, including long goals pursued by a single agent.
- Make steering discoverable and progress toward outcomes useful.
- Use the TDD skill when implementation begins, following the agreed spec and public testing seams.

Background: [project handoff](HANDOFF.md), [existing specification](GOAL-CONSOLE-SPEC.md), and [current controls](GOAL-CONSOLE.md).

## Q1 — Visible work breakdown

Should a large goal show named subgoals or phases, their dependencies, the current approach, what changed, and what comes next—even when one agent does everything?

**Recommendation:** yes. Make meaningful work units visible, with technical commands expandable beneath them.

- [X] Yes — show meaningful work units. **Recommended.**
- [ ] Improve summaries on the existing agent cards only.
- [ ] Use my custom answer below.

**My answer, changes, or notes:**
Yes, but let's talk about how exactly will we achieve this before implementing (and even before implementing the tests for it)
<!-- Write here. -->

## Q2 — Execution and delegation

Should this iteration focus on explaining and steering the work while Codex continues to choose how to delegate it?

**Recommendation:** yes. Show actual sequential and parallel work, then assess whether scheduling itself needs to change.

- [X] Yes — keep Codex in charge of delegation. **Recommended.**
- [ ] Also change how work is split and delegated.
- [ ] Use my custom answer below.

**My answer, changes, or notes:**

<!-- Write here. -->

## Q3 — Where semantic context comes from

Should we primarily reuse plans and public commentary, plus brief structured updates from the working agent, with an optional small model for gaps?

**Recommendation:** yes. Record updates at meaningful transitions and make any extra model usage explicit and bounded. If we include an additional model, its trigger, role and budget will need decisions in a later round.

- [X] Yes — agent updates plus an optional small model. **Recommended.**
- [ ] Agent updates only; no additional model.
- [ ] A small model should maintain the overview automatically.
- [ ] Use my custom answer below.

**My answer, changes, or notes:**

<!-- Write here. -->
Yes, but once again let's be careful about this small model, it needs to work as best as it can without it and then it's like an extra thing that needs to be explicitly enabled either globally or on per project basis
## Q4 — Steering discovery

Should selecting work immediately show a clearly labeled **Steer** action, the actual recipient, and whether live input is available?

**Recommendation:** yes. Keep the draft, send/queue/stop actions, and delivery status together so steering is understandable from the selected work.

- [X] Yes — make steering visible on selection. **Recommended.**
- [ ] Keep it in a separate panel with better hints.
- [ ] Use my custom answer below.

**My answer, changes, or notes:**

<!-- Write here. -->

## Anything else

Examples of confusing sessions, desired behavior, or additional constraints are welcome here.

<!-- Write here. -->
To which extent can we interact with the cached stuff that LLMs/agents have?
## What follows your answers

Resolve any remaining design questions, confirm shared understanding, then use `to-spec` and `implement-spec` to specify and build the agreed changes with vertical TDD.

## Round 2 — How this works

Your Round 1 answers establish visible work units, Codex-owned delegation, a strong no-model baseline, explicit global/project opt-in for any helper model, and visible steering. These questions concern the mechanism in [the proposal](SEMANTIC-CONTEXT-DESIGN.md).

### Q5 — Outcomes and work

Should we use the proposal's two connected views: evidence-backed outcomes for what “done” means, and work items for the steps being attempted, initially populated from existing plans and then enriched by explicit updates?

**Recommendation:** yes. Keep work status, ownership, source and update time visible. Show dependencies and outcome links only when explicitly recorded; a completed plan step alone does not verify an outcome.

- [X] Yes — use this outcome/work model. **Recommended.**
- [ ] Use my custom answer below.

**My answer, changes, or notes:**

<!-- Write here. -->

### Q6 — Getting updates from the working agent

Should a visible **Enable tracking** action prepare one short, editable instruction for the selected goal's coordinator, sent through the normal guidance/queue flow, asking for compact updates only at meaningful transitions?

**Recommendation:** yes. Reuse the agent's existing context, publish through the local state interface, and retain native plans/public commentary as the fallback. This has some token cost when the agent reports; simply watching remains free of model calls. Per-goal setup is the initial proposal.

- [X] Yes — enable cooperative tracking per goal. **Recommended.**
- [ ] Prefer a reporting convention configured once per project.
- [ ] Rely entirely on existing records, without requesting agent updates.
- [ ] Use my custom answer below.

**My answer, changes, or notes:**
Yes, but also doing a reporting convention for specific projects can be nice to potentially conserve more tokens
<!-- Write here. -->

### Q7 — The default view inside a goal

Should opening a goal show its work items and verified progress first, with owners, purpose/next step and visible steering, plus clearly labeled actions for **Agents**, **Commands**, and **Replay**?

**Recommendation:** yes. Keep the global fleet compact, expand details on demand, and make the distinction between inspecting work and opening recorded replay explicit.

- [X] Yes — make work the default goal view. **Recommended.**
- [ ] Keep agents as the default and add a separate Work view.
- [ ] Use my custom answer below.

**My answer, changes, or notes:**

<!-- Write here. -->

### Q8 — When to build the optional helper

Should we complete and try the full no-model flow first, then choose the helper's exact role, trigger, model and budget based on the remaining gaps?

**Recommendation:** yes. Preserve your requirement that any helper must be explicitly enabled globally or per project; do not add model calls before its behavior and cost are agreed.

- [X] Yes — build and assess the no-model flow first. **Recommended.**
- [ ] Specify and implement the optional helper in this iteration too.
- [ ] Use my custom answer below.

**My answer, changes, or notes:**

<!-- Write here. -->
