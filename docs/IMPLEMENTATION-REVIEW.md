# Implementation review

Your five questionnaire answers are recorded and understood. This is the last review before new tests and product implementation.

## What the spec now includes

- A global terminal graph on launch, with one-key access to a searchable graph of history.
- Evidence-based completion criteria and progress; unknown progress stays unknown.
- Targeted live guidance, later queueing and stopping, while keeping native Codex usable.
- Meaningful task descriptions and inspectable Bash commands; routine file reads/writes remain concise.
- Continuous work with as few interruptions as possible and optional checkpoints.
- Questions and partial answers that persist across navigation and restart.
- Generated-document links and an open-in-VS-Code action using your installed MD Reader plugin.
- No model calls for passive discovery/rendering; criteria assistance is explicit rather than a hidden summarizer.

The full draft is [GOAL-CONSOLE-SPEC.md](GOAL-CONSOLE-SPEC.md). Its dependency-aware work is in [GOAL-CONSOLE-TICKETS.md](GOAL-CONSOLE-TICKETS.md).

## Required test-scope agreement

The TDD skill requires agreement on public testing seams before writing any new test. These are the two proposed seams:

1. **The console as you use it:** send session/runtime updates and keyboard/mouse input through the public console interface; check the rendered result and external actions. This catches the graph, history/search, progress, persistent drafts/questions, feedback recipient and document-open behavior. It misses terminal-specific font quirks and model quality.
2. **The executable and its external connections:** run the actual command against isolated session/state fixtures and a local Codex protocol peer; exercise restart, legacy commands and the editor bridge's public launch interface. This catches command wiring, persistence, control-protocol failures and exact file opening. It misses compatibility with every possible future Codex or VS Code release.

Tests will cover behavior at these seams, not private functions or internal call counts. Existing tests may continue to run; the baseline of 216 existing tests already passes. No new tests have been written yet.

- [ ] Yes — approve these two testing seams and the proposed spec. **Recommended.**
- [X] I have changes below.

**Changes or notes:**

<!-- Write here. -->
I do approve, however why do we miss model quality?

## Workflow setup

Your fork's GitHub Issues setting is disabled. To follow the requested spec/ticket workflow, the proposed setup is to enable Issues on `wagar-meilech/zoetrope`, publish the approved spec and its tickets there with `ready-for-agent`, and use one integration PR to close them. That has not been done yet.

- [X] Yes — use the fork's GitHub Issues and integration PR workflow. **Recommended.**
- [ ] Keep the spec and tickets as local Markdown instead.

You can check the boxes here and save, or reply **“approve both”** in chat. You can also edit the spec directly before approving it.

Also there was some local ticket thing we could use that was not markdown I think that is good too, but feel free to change those github settings in-browser 
