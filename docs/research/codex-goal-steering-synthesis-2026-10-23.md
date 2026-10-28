# Goal-oriented steering interfaces for Codex

Research snapshot: 23 October 2026. This extends the [initial terminal interface research](codex-agent-interfaces-2026-10-23.md). Findings were checked against project documentation and selected source/change records; the products were not installed or tested end to end.

The desired interface is a terminal graph organized around a goal, with readable outcome/activity labels, direct feedback while work runs, optional shell-command details with a short purpose, and low additional model usage.

## Finding

**Ordewell is the closest newly found existing terminal workflow for goal-oriented control. Zoetrope remains the strongest visual starting point for the exact desired graph.** No inspected product or fork establishes all of terminal node graph, whole-goal checklist progress, in-view live steering, and consistently plain-language purpose labels.

| Project | Useful capability | Material mismatch |
| --- | --- | --- |
| [Ordewell](https://ordewell.ai/docs) | Describe a goal, edit a task plan, open running task conversations, send feedback or interrupt, and revise the plan | Task/tree-oriented orchestrator rather than a Zoetrope canvas; adopts its own task/worktree workflow |
| [Open Agent View](https://github.com/xhluca/open-agent-view) | Terminal dashboard with managed Codex inline replies, approvals/input, and stop controls | Session overview; no verified goal-progress graph |
| [Codex Goal Progress](https://github.com/Ezra-Y/codex-goal-progress) | Checklist-based goal progress calculated locally; model supplies checklist and evidence updates | Codex Desktop on Apple Silicon; not a terminal agent graph or steering interface |
| [Herdr Sheppard](https://github.com/bloodf/herdr-sheppard) | Zoetrope fork with fleet management, prompt delivery and confirmed interruption | Busy agents reject prompts; graph and manager controls are separate; no goal-percentage model |

Ordewell's current documentation describes opening a task to read its live log and type to it, with a send-now action and interruption. Its changelog describes a structured Codex app-server connector and send-now handling that can interrupt and replace the active input. This is stronger control than a passive observer, but should not be described as universally non-disruptive mid-turn steering. The plan can be edited locally; structural edits during execution wait for a batch boundary. Its optional supervisor uses planner turns, including the default Watch setting; turning supervision off avoids those extra monitoring turns. [Docs](https://ordewell.ai/docs), [changelog](https://github.com/ordewell/ordewell/blob/main/CHANGELOG.md)

Open Agent View exposes control only for capabilities available on the particular session. Its managed Codex controller owns an app-server connection and tracks exact thread/turn identity. Externally discovered sessions do not automatically grant the same controls. The terminal Peek interface supports conversational reply/steer for supported managed sessions. [CLI reference](https://github.com/xhluca/open-agent-view/blob/main/docs/cli.md), [architecture](https://github.com/xhluca/open-agent-view/blob/main/docs/architecture.md)

Codex Goal Progress separates progress from token/time accounting. It weights completed checklist items locally and holds at 95% while the native goal is still active, then shows 100% on native completion. Evidence is supplied by the current model; deterministic calculation is not independent proof that the checklist covers the user's actual goal. This is a useful model to learn from, not an independently calibrated ETA. [Architecture](https://github.com/Ezra-Y/codex-goal-progress/blob/main/docs/ARCHITECTURE.md)

## Fork audit

The fork audit enumerated 78 public Zoetrope forks at the metadata level, then inspected a bounded set of divergent or otherwise relevant candidates. No prominent, established fork emerged; the highest-starred fork had three stars in the snapshot. The meaningful control-oriented discovery was `bloodf/herdr-sheppard`.

Sheppard is worth inspecting as control integration work, but it permits prompt delivery only to idle/done, interactively ready agents. Working agents require confirmed interruption first. It does not deliver the desired select-a-busy-node-and-steer behavior. [Fork network](https://github.com/furkankly/zoetrope/forks), [manager control and recovery boundaries](https://github.com/bloodf/herdr-sheppard/blob/main/manager/README.md)

Detailed inventory and checked candidates: [fork audit](zoetrope-forks-steering-2026-10-23.md). Additional alternatives and source caveats: [steering landscape](codex-steering-alternatives-2026-10-23.md).

## Recommendation if adapting Zoetrope

This is a proposed design, not existing functionality.

1. **Organize the graph around outcomes.** Show Goal → milestones → current work, with agents assigned to that work. Preserve agent lineage in an expandable secondary view. Finishing three agents is not equivalent to satisfying three success criteria.
2. **Make the selected node actionable.** An always-available feedback input targets the selected work. Expose delivery state and whether guidance went to that worker or its coordinator. Distinguish steering, next-turn queueing, and interruption; do not label one as another.
3. **Keep the top-level language human.** Show “Checking reconnection” rather than a tool identifier. Tool-name mappings, existing task titles and file-action parsing can produce many labels locally. Keep shell commands in expandable details with the agent's stated short purpose.
4. **Show completion against criteria.** Use “3 of 5 outcomes verified,” optional weighted checklist percent, the current milestone, and unresolved work. Record scope revisions explicitly. Avoid treating elapsed time or token consumption as completion.
5. **Keep the observer inexpensive.** Reuse existing plan updates and commentary. Have the working agent supply one short purpose at a meaningful change of activity, rather than running a second summarizer for every tool call. Polling and redrawing stay local.

An illustrative display:

```text
Goal: Make reconnection reliable
Progress: 3 of 5 outcomes verified
  ✓ Understand the failure
  ✓ Reproduce it reliably
  ✓ Implement reconnection
  ● Check behavior after sleep
    Why: Confirm the connection recovers after the laptop wakes
  ○ Review the finished change

Feedback to “Check behavior after sleep”: ...
```

A command detail might read `pnpm test reconnect` with the purpose “Check that the fix survives a dropped connection.” That is an illustrative label, not a command executed during this research.

### Why steering needs more than a visual change

Zoetrope deliberately reconstructs state from append-only transcripts and is read-only. A transcript can describe an agent without providing a live control endpoint. [Zoetrope architecture](https://github.com/furkankly/zoetrope/blob/main/docs/ARCHITECTURE.md)

The clean Codex-specific approach is a supported live app-server connection with exact thread/turn identities, using the same managed/shared session that the terminal uses. Official OpenAI documentation exposes active-turn steering, interruption, goal state, plan-step updates and token events. The connection/ownership work needs verification before claiming control of an arbitrary already-running Desktop or CLI session. [Official App Server documentation](https://learn.chatgpt.com/docs/app-server), [CLI connection options](https://learn.chatgpt.com/docs/developer-commands)

Keep the transcript parser/replay model separate from the live control client. Browsing an old replay should never accidentally send guidance to the current run. This separation is an implementation recommendation, not a claim about the present upstream code.

### Token budget implications

Local label mappings, stored-title rendering, checklist arithmetic and terminal drawing do not require model inference. New explanations and status updates do consume model tokens and can increase later context. A short purpose per meaningful batch should be much cheaper than repeated whole-session summarization, but the size of that overhead has not been benchmarked here. Reusing existing commentary is preferable when it already provides a clear explanation.

The user-facing purpose should be attributed as the agent's stated intent; avoid reconstructing hidden reasoning or inventing a reason from a command alone.

## Other useful work

- [Herdr Reviewr](https://github.com/persiyanov/herdr-reviewr) adds terminal diff comments that can be sent back to Codex. Useful steering during code review, not a goal-progress graph.
- [Supergoal](https://github.com/Lancelot-Xie/Supergoal) provides dependency-aware subgoals, evidence and a static/read-only dashboard. Its human control workflow does not make the dashboard itself a live steering interface.
- [Rungraph](https://github.com/fayzan123/rungraph) supports inspecting runs and asking the existing agent questions about graph nodes; it does not establish direct live graph steering. Asking the agent adds model work.
- [Keel](https://github.com/codejunkie99/keel) provides graph controls in a substantial desktop workspace. Its selected-subagent steering/stop path targets the parent run, a material distinction from direct child control. See the source analysis in the steering landscape note.
