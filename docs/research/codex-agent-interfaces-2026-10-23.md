# Codex agent interfaces: terminal-first research

Checked 23 October 2026 against project documentation, selected source files, and official OpenAI documentation. This is source research, not hands-on compatibility testing. User priorities: terminal-based interface; live expandable graph/tree; feedback during execution; credible goal progress; low token overhead. A small companion window is acceptable.

## Recommendation

Start with Codex CLI in one terminal pane and **Zoetrope** in another for the actual live agent graph. Use **AgentHUD** instead when an expandable overview across many projects and sessions matters more. Both observe existing local logs; the expected additional model-token cost of passive visualization is zero. That is an architectural inference, not a measured benchmark. Feedback stays in Codex's interactive terminal.

The main missing piece is a validated percentage of the complete user goal. A playback position, context-window gauge, token budget, or count of completed plan steps measures a different quantity. None of the terminal-first tools reviewed documents a generally reliable remaining-work estimator.

## Shortlist

| Interface | Verified role | Intervention | Progress limitation |
| --- | --- | --- | --- |
| [Zoetrope](https://github.com/furkankly/zoetrope) | Terminal graph of a Codex session, nested subagents and tools; live follow, node inspection, pan/zoom; CLI and desktop session logs | Read-only; return to the Codex pane for instructions | Timeline is replay position; pausing it pauses the visualization, not the running agent |
| [AgentHUD](https://github.com/neochoon/agenthud) | Terminal tree across projects, sessions and children, with selected-node live activity | Read-only monitoring; use original CLI | Percentage is context-window occupancy |
| [Agent Deck, asheshgoplani](https://github.com/asheshgoplani/agent-deck) | Tmux-based session manager with grouping, live status and session attachment | Attach to Codex and steer there; manager also offers message/queue commands | Session organization does not establish an automatic graph of native Codex subagents |
| [Native Codex CLI](https://learn.chatgpt.com/docs/developer-commands) | `/agents` command center and `/subagents` thread switching | Enter injects instructions during the turn; Tab queues them | Documented controls are status/navigation rather than a graphical whole-goal percentage |

Zoetrope quick trial, if installing later: `brew install furkankly/tap/zoetrope`, then run `zoe --provider codex .` in the project's second terminal pane. A file or session ID can identify a specific run. Its Herdr integration can toggle the live graph over the active agent pane. Log-format changes can affect compatibility; the project explicitly describes the transcript formats as internal. [Project README](https://github.com/furkankly/zoetrope)

AgentHUD can be tried with `npx agenthud`. Enter expands/collapses sessions; `t` follows the newest live subagent. Its Codex provider explicitly links child rollout files with `parent_thread_id`, rather than merely grouping unrelated sessions. The ordinary HUD reads files; `agenthud summary` separately invokes an LLM and therefore consumes model tokens. [Keyboard and feature reference](https://github.com/neochoon/agenthud/blob/main/FEATURES.md), [Codex schema and context-gauge calculation](https://github.com/neochoon/agenthud/blob/main/docs/schemas/codex-session.md), [Codex provider source](https://github.com/neochoon/agenthud/blob/main/src/data/providers/codex.ts), [README](https://github.com/neochoon/agenthud)

Agent Deck is the specific Go/tmux project linked above: multiple unrelated projects share the name. Its manager can be useful for many independent agents, but its extra conductor orchestration is model work and should not be equated with a free monitor. Codex cost collection is described as output parsing and untested, so do not select it for authoritative Codex spend accounting. [README](https://github.com/asheshgoplani/agent-deck)

## Native Codex control and progress signals

Official OpenAI documentation confirms `/agents` browses chats on the connected shared local or remote server; `/subagents` switches to a child thread to inspect or continue work. During work, Enter steers the active turn and Tab queues a follow-up. [Developer commands](https://learn.chatgpt.com/docs/developer-commands)

`/goal` keeps a durable objective attached to long-running work; measurable completion criteria and checkpoints make progress more assessable. It is not documented as an automatic calibrated percent-complete estimator. [Long-running work](https://learn.chatgpt.com/docs/long-running-work)

The Codex App Server provides `turn/plan/updated` (steps with pending/in-progress/completed states), `thread/tokenUsage/updated`, collaboration tool items with sender/receiver/new-thread IDs, and `turn/steer`. A lightweight client could draw a tree and milestone bar directly from those events without repeatedly asking an LLM for status. This is an implementation inference; it is not a claim that an existing client has shipped the full desired interface. [App Server documentation](https://learn.chatgpt.com/docs/app-server)

## Closest workflow-oriented progress-bar alternative

[claude-dynamic-workflows-codex](https://github.com/scasella/claude-dynamic-workflows-codex) runs workflows on the local Codex backend, despite its name. It supplies an expandable browser DAG/tree with phase progress bars, tokens and timings, plus human decision gates and stateful worker steering. The standalone runner does not require Claude Code. It also has a live terminal ASCII map: `node runner/bin/map-run.js <run-directory> --watch`.

This requires adopting/authored workflows rather than watching arbitrary existing Codex sessions. Browser phase bars describe the workflow's known work, and human input is exposed at declared `human()` checkpoints; do not assume arbitrary click-to-steer controls on every running node. It is a more involved fit than adding a passive terminal viewer. [README](https://github.com/scasella/claude-dynamic-workflows-codex), [terminal map source](https://github.com/scasella/claude-dynamic-workflows-codex/blob/main/runner/bin/map-run.js), [authoring reference](https://github.com/scasella/claude-dynamic-workflows-codex/blob/main/references/authoring.md)

## Token implications

Rendering existing events locally need not consume any model tokens. Model-generated status summaries, AI conductors, and additional workers do. For low overhead, keep the observer passive and update a compact plan only at meaningful checkpoints. If showing a percentage, label it as completed milestones, separate verification from implementation, and expose scope changes instead of implying an exact ETA.

OpenAI explicitly notes that subagents consume more tokens because each performs its own model and tool work. The viewer's overhead and the underlying team's cost are separate. [Subagent documentation](https://learn.chatgpt.com/docs/agent-configuration/subagents)

More third-party findings and source checks: [companion research note](codex-agent-visual-interfaces-third-party-2026-10-23.md).
