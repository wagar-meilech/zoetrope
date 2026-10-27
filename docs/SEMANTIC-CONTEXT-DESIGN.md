# Semantic context — mechanism for discussion

Status: accepted through the completed [questionnaire](SEMANTIC-CONTEXT-QUESTIONS.md) and “Answered, proceed” on 27 October 2026. The implementation contract is [SEMANTIC-CONTEXT-SPEC.md](SEMANTIC-CONTEXT-SPEC.md). The layout below remains illustrative.

## What your answers establish

- Show meaningful work within a large goal, including work performed by one agent.
- Keep Codex responsible for execution and delegation.
- Make the core useful without an additional model.
- Any additional model must be explicitly enabled globally or per project.
- Show steering availability and its recipient when work is selected.
- Agree on the mechanism before writing tests or implementation. Then use the TDD skill.

## The proposed mechanism

Keep **outcomes** and **work** connected but distinct. An outcome describes what must be true when the goal is achieved. A work item describes an attempt to achieve it. An agent owns work; completing its plan does not by itself verify an outcome.

For example, a single coordinator investigating steering could produce this view:

```text
Goal: Make steering understandable
Outcomes: 1 of 3 verified

  ✓ Inspect current navigation       Coordinator · reported complete
  → Expose available steering        Coordinator · in progress
  ○ Verify input reaches its target  waits for available steering

Selected: Expose available steering
Purpose: Show what input this recipient can accept before sending.
Next: Check the unavailable-child explanation.
Evidence: No verification recorded for this outcome yet.
Source: Agent update · timestamp shown

[Steer coordinator]  [Agents]  [Commands]  [Replay]
```

The dependency in this example would need to be explicitly recorded. A sequence in a plan alone would not establish a blocking relationship. The example's labels and layout are illustrative, not implemented controls.

### 1. Reuse existing records immediately

Zoetrope already imports native plan steps and their reported statuses. Render those steps as work, including pending/in-progress/completed, producer and source. Reuse public commentary and recorded task descriptions for readable context. Carry timestamps so older narration is distinguishable from current activity.

A session without a published work structure still gets the best available native plan and public updates. Missing dependencies, purpose or outcome links remain visibly unknown. Generic file operations cannot reliably reveal an unstated purpose.

Native plan steps currently lack stable step identifiers. Treat a changed native plan as a source revision; do not pretend that text similarity proves a step retained its identity. Persistent work items published explicitly can have stable identifiers.

Where the installed runtime supports reading a native goal's objective and status, those could also populate the heading as attributed native reports. This is not currently imported by Zoetrope, and native goal completion would not substitute for evidence-backed outcome verification.

### 2. Let the working agent publish a small work record

Extend the existing local `zoe state` interface so an agent can record a work item's stable ID, short title, owner, reported status and optional outcome links. Dependencies, a short purpose, next action or blocker are included when known. Updates carry their source and revision; later edits update that work item instead of creating a new graph node each time.

Outcome links include the goal revision, so a scope change can flag links that need review. Preserve earlier evidence instead of silently crediting a renamed or replaced outcome. Concurrent updates use the existing atomic store with revision checks; repeated updates should not duplicate work.

The working agent already has the task context. Asking it for a concise update avoids sending the entire transcript to another model. This does consume some agent output/tool tokens; passive viewing of those saved records consumes none.

Use updates at meaningful transitions: establishing or revising the plan, starting or finishing a work item, encountering a blocker, changing scope, or recording verification. Do not request an update for every file read or command, or on a timer. Batch related changes in one compact operation and return a short acknowledgment.

### 3. Make cooperation an explicit action

Provide a visible **Enable tracking** action for a selected goal. It prepares a short, editable instruction for the actual coordinator, including the exact local record and compact update interface. Sending uses the existing explicit guidance/queue flow and displays its delivery state.

This would establish an ongoing reporting convention for that goal. It would not replace the running session, change its model or permissions, or require another model to monitor it. Acceptance by Codex does not guarantee that the agent will maintain the record; show the last reported update and any gaps honestly.

An ordinary historical session remains inspectable without enabling cooperation. Provide reusable opt-in project reporting guidance alongside the per-goal action. A small conditional instruction pointer loads details for tracked work; the operator chooses which projects install it.

### 4. Show work and progress together

Keep the fleet overview compact. Opening a goal shows its work first, with owners and explicitly recorded dependencies. Provide clearly labeled access to agents and recorded replay. Expanding a work item reveals its context and associated commands/documents without turning every tool invocation into a graph node.

Show verified outcomes alongside the current work, including the evidence for each verification. A reported completed work item may still have an unverified outcome. Existing outcome evidence and scope revisions remain authoritative for the goal's verified progress.

Selecting a work item also identifies its owner. Steering names that actual agent and shows whether guidance, queueing or stopping are available. An unavailable child has an explicit coordinator-selection action; sending never silently changes recipients.

### 5. Keep the Rust implementation bounded

Extend the existing provider projection, local work records and console; retain the portable replay core. Use incremental reads, compact cached summaries, bounded work/history retention, lazy detail loading and the existing background IO workers. Rendering performs no transcript scans or model calls.

The existing native plan is a fallback source, not a reason to overwrite an explicitly maintained work record. Retain provenance when the operator edits information or scope changes. Measure representative histories and idle behavior through the public console rather than assuming the implementation is efficient.

## What can we use from an agent's “cache”?

| Material | What can be reused | Limit |
| --- | --- | --- |
| Saved plans, public messages and tool results | Read supported local records or fetch persisted thread items without generating another response. | Saved history is not an exact view of the current model input. Our bounded index can also omit older records until explicitly loaded. |
| Explicit notes, documents and handoffs | Link or read user-visible files; reference a small relevant note when steering. | A file existing on disk does not establish that the active agent has read it or still has it in context. |
| Compaction state | Observe that compaction occurred; reuse a readable handoff only when one is actually exposed. | The Responses API's encrypted compaction item is opaque, and Codex's documented compaction event is not a readable summary. |
| Provider prompt/KV cache | Inspect reported cache-hit/token counts where available; continued input to the existing session may benefit from cache reuse. | It is not a queryable semantic store or shared memory for a different model. The documented interface does not expose its tensors as readable context. |
| Zoetrope's own metadata cache | Retain the compact work/goal projection and update it incrementally. | It represents observed records, not the model's internal knowledge. |

The practical opportunity is to reuse the **working agent's existing conversation**: ask it to publish a small state update from what it already knows. This still requires generation. Cache reuse can reduce processing cost when a matching prefix is available; it does not make a new answer free or guarantee a cache hit.

This is grounded in the local Codex 0.164 schema exports and current official documentation. A schema advertises a shape; compatibility still needs verification against the installed runtime before the app claims support.

Sources:

- [Codex App Server](https://learn.chatgpt.com/docs/app-server): persisted history reads, plan updates, token usage and compaction events.
- [OpenAI prompt caching](https://developers.openai.com/api/docs/guides/prompt-caching): KV-state reuse, prefix matching and cache usage.
- [OpenAI compaction](https://developers.openai.com/api/docs/guides/compaction): opaque encrypted compaction items.

## Optional model assistance

The no-model flow above should stand on its own. A helper could later suggest an activity-to-work-item match, a missing outcome link, or a short explanation from available public records. Such suggestions would be labeled and would not verify outcomes or control agents.

Your global/project opt-in requirement is settled. A project-specific disable should override a global enable, and the effective setting should be visible. The helper's exact role, provider/model, trigger and spending limit are still undecided. Enabling it may introduce a remote data path, depending on the chosen provider; that must be explicit in its configuration.

Your Q8 answer defers implementing the helper until the no-model flow has been completed and assessed.

## Validation after design agreement

Use the existing approved public console and executable/external-connection seams. They can exercise visible native plan statuses, explicit work updates across restart, stable selection, outcome evidence, and recipient-specific controls. They do not establish that an agent's explanation is correct or that every runtime version supports the same operations.

Implementation follows vertical TDD: one meaningful failing behavior, the minimum change to make it pass, then the next behavior. The accompanying specification and task graph carry the implementation requirements.
