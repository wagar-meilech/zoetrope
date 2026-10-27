# Codex interfaces focused on steering, goal progress, and readable activity

Research checked 23 October 2026. Primary project documentation and selected source were inspected; software was not installed or run. Scope: new alternatives beyond the earlier Zoetrope/AgentHUD/Agent Deck/CodexMonitor/workflow-runner comparison. Terminal operation remains the preferred surface.

## Assessment

The requirements exist in separate products: **Ordewell** has the strongest terminal goal/task control; **Open Agent View** has concrete inline Codex steering in a terminal; **Herdr + reviewr** makes intervention easier while preserving the actual CLI; **Codex Goal Progress** implements checklist-derived goal percentages; **Keel** has graph-node steering but is a full native workspace. No inspected product establishes the complete desired combination of a Zoetrope-like terminal graph, a whole-goal acceptance view, simple labels, and direct control of every native Codex child.

## 1. Ordewell: strongest goal/task-centric terminal workflow

One user goal becomes a typed editable plan with tasks, subtasks, dependencies, runner assignments, and task statuses. The TUI shows conversation beside the plan, and opens each task's live log/chat. Enter sends a message that waits for the current step; Ctrl+S sends now; Ctrl+X interrupts the runner turn. Checkpoints accept user directions. Direct plan edits need no model round trip; prose plan edits do. [Current user documentation](https://ordewell.ai/docs)

The changelog confirms structured Codex execution over `codex app-server`, nested subtask expansion, and slimmer title-first plan rows. Its send-now behavior interrupts/replaces the running turn rather than promising that an arbitrary tool is instantly undone. A default Watch supervisor consumes extra planner turns on interesting events; it can be switched off. This is a new orchestrated workflow, not an observer attached to any existing Codex session. Its dependency-aware task tree is not evidence of a Zoetrope-style node canvas. Exact Codex steering RPC code was not successfully fetched, so only the documented behavior and connector architecture are verified. [Changelog](https://github.com/ordewell/ordewell/blob/main/CHANGELOG.md)

## 2. Open Agent View: actual inline terminal steering

Open Agent View is a Rust TUI with a session list, status groups, native-session switching, and inline inspection. Its Codex feature matrix includes reply, approvals/input, stop, and archive/delete. Space opens Peek; typing and Enter sends an owned-provider reply/steer. Controls require a session it owns and a supported capability. This is stronger intervention than a passive log viewer, but it does not document a goal checklist percentage or nested graphical agent map. [README](https://github.com/xhluca/open-agent-view), [keyboard/control reference](https://github.com/xhluca/open-agent-view/blob/main/docs/cli.md)

Its architecture uses a detached Codex App Server on a private Unix socket, retaining exact thread and active-turn identities with verified process ownership. External history remains read-only, rather than granting live control merely because a transcript exists. This is a useful example architecture for adding reliable control to Zoetrope. Ordinary discovery/rendering does not itself invoke a model; zero extra model tokens for that observation is an inference from this design. [Architecture](https://github.com/xhluca/open-agent-view/blob/main/docs/architecture.md)

## 3. Herdr + reviewr: best companion to an unmodified Codex terminal

Herdr keeps agents in real terminal panes and offers direct attachment, named targets, state rollups and notifications. `agent prompt` can submit to an already-working agent; key commands can interact with native controls. This is terminal input routing, not a Codex-native graph controller or proof of turn-specific delivery. Custom agent names and display-only summary metadata support readable labels, but no automatic semantic goal decomposition is documented. [Agent controls](https://herdr.dev/docs/agent-automation/), [labels and status](https://herdr.dev/docs/agents/)

[herdr-reviewr](https://github.com/persiyanov/herdr-reviewr) adds a terminal diff-review pane: annotate lines/ranges and send comments into Codex's input. It supports uncommitted, branch, last-turn, and commit scopes. Send requires Herdr and a live agent, and refuses permission prompts; it does not send automatically. The README says comments are put into the input, so avoid claiming guaranteed automatic mid-turn submission without a hands-on test. It is an intervention aid rather than goal progress or an agent graph. Reading and annotating add no model calls; the explicit feedback, once submitted, uses ordinary agent tokens. [Project documentation](https://github.com/persiyanov/herdr-reviewr)

## 4. Codex Goal Progress: the missing acceptance-progress model exists

This plugin adds objective/overall progress beside the native desktop Goal. It requires Codex Desktop on Apple Silicon; it is not a terminal frontend. The current Codex model prepares/reuses a checklist and updates evidence via local MCP tools; a local Helper validates updates and computes the display. [README](https://github.com/Ezra-Y/codex-goal-progress)

The calculation uses weighted required objectives and completed checklist ratios, caps the display at 95% while the native Goal remains active, and reaches 100% when that Goal completes. Tokens, elapsed time and command count remain separate. This is deterministic arithmetic over model-maintained evidence, not independent validation that an arbitrary goal is truly satisfied. Scope can be revised without erasing completed facts. It is promising code/design to borrow for a Zoetrope adaptation; creating/updating checklist evidence costs some same-agent output, while rendering/recalculation is local. [Architecture](https://github.com/Ezra-Y/codex-goal-progress/blob/main/docs/ARCHITECTURE.md), [checklist and scope contract](https://github.com/Ezra-Y/codex-goal-progress/blob/main/skills/goal-progress/references/checklist-and-scope.md)

## 5. Keel: graph-node steering, with an important child-target limitation

Keel's macOS GPUI workspace has a graph of active sessions, child task nodes, and shared files. It supports Codex through ACP. It is a full workspace with editor/transcript/terminal/settings and currently directs users to build from source; the README says downloadable public releases are not available. This is materially more GUI-oriented than the requested terminal companion. [README](https://github.com/codejunkie99/keel)

The graph source implements steer and stop, but a selected subagent is addressed by sending the parent a message prefixed with the child task's label. Stop also targets the parent chat. Thus graph selection does not establish direct isolated child steering/termination. No goal-completion percentage appears in the inspected graph module. [Agent graph source, especially `target`, `steer`, and `stop`](https://github.com/codejunkie99/keel/blob/main/crates/ui/src/agent_graph.rs)

## 6. TUICommander: readable intent, progress journal, native terminal control

Despite the name this is a desktop terminal workspace. It recognizes Codex terminals and provides per-terminal controls. It can show short declared `intent:` text as tab titles. Its Progress surface is explicitly a journal of intent, delegation, messages, done and blocked events; it is not whole-goal percentage tracking. It records one-sentence outcome reports from the agent, so readable labels need not require a separate summarizer. [Agent support](https://tuicommander.com/docs/user-guide/ai-agents.html), [project progress](https://tuicommander.com/docs/user-guide/project-progress.html), [settings](https://tuicommander.com/docs/user-guide/settings.html)

There is version drift between cached search snippets and opened documentation around the exact Progress schema. Treat the current product as an idea source for intent labels and concise outcome events, not a verified stable API for integration. Goal graph and direct native-child control were not established.

## Other candidates checked

- **[Rungraph](https://github.com/fayzan123/rungraph):** live browser graph, tool grouping, human intervention nodes, and agent-generated answers that highlight exact graph nodes over MCP. It can reuse the agent's narration immediately before tool calls. Its MCP surface reads/queries/focuses the graph rather than controlling the running agent. Interactive Q&A consumes the user's agent tokens; the README notes a large full graph can exceed 20,000 tokens, recommending targeted queries. Useful inspiration for readable tool explanations, less useful for low-token steering.
- **[AgentDeck, puritysb](https://github.com/puritysb/AgentDeck):** upstream of leonj1/agentdeck. Terminal monitoring plus many hardware/native surfaces. The TUI is primarily sessions, aquarium animation, usage and timeline. Immediate terminal steering remains managed-session-only, and generic observed sessions may offer display-only prompts. No goal checklist or terminal DAG was established. [TUI reference](https://github.com/puritysb/AgentDeck/blob/master/docs/tui-dashboard.md), [README control boundary](https://github.com/puritysb/AgentDeck)
- **[propr](https://github.com/integry/propr):** long-running goals with correction/control features, but a self-hosted GitHub/PR orchestration web application. It does not fit the requested lightweight terminal surface closely enough to prioritize.

## Token-efficient readable activity: implementation inference

The cheapest path is to reuse the agent's existing commentary/intents, map raw tool names locally to verbs, and reveal technical arguments only on demand. Keep a bash command visible under one short sentence explaining its purpose. If the agent did not state a purpose, show an observable label rather than inventing its intent. Request a compact title/purpose only when meaningful work changes; avoid a second model repeatedly rereading the full transcript. These are design conclusions, not measured overhead claims.

For progress, acceptance outcomes should be the nodes that earn completion credit; tool calls are the evidence/details beneath them. Keep the user's goal and current scope visible, allow scope edits, and distinguish a saved feedback message from accepted/delivered guidance. The existing projects provide separate examples for each piece, but adapting Zoetrope would need a real control connection, not just a prettier log reader.
