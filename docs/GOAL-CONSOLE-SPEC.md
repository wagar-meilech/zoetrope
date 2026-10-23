# Zoetrope goal console

Status: approved on 23 October 2026 through the user's implementation-review replies. Both public testing seams and the fork's GitHub issue/PR workflow are approved. Product direction follows the answered customization questionnaire.

## Problem Statement

The user runs Codex work across projects and wants one terminal graph to understand and steer it. Zoetrope currently displays one session and its subagents. Its overview camera does not reveal other sessions. Tool counts, token usage and the playback slider do not show completion of the user's goal. Feedback requires returning to a separate interface, temporary questions disappear, and documents created by agents are difficult to find again.

The user wants a global view of ongoing work, a one-key switch to searchable history, honest progress toward outcomes, persistent targeted feedback and questions, and direct access to generated documents. Ordinary Codex access must remain usable. Passive monitoring should not invoke a model, and routine file reads/writes do not need elaborate semantic descriptions.

## Solution

Opening `zoe` shows a fleet graph grouped by project, goal/session and agent. Active work and requests needing attention appear first. One key switches between current work and all known history. Search filters the same graph. Expanding a session reveals its agents; opening its detail view retains the existing single-session graph and replay tools.

The selected goal or agent has a persistent feedback composer with separate guidance, queue and stop actions. A local Codex connection controls the original live thread when supported, without creating a replacement run or changing its access policy. Unsupported sessions remain visible with an honest explanation of which actions are available. Native Codex continues to work alongside Zoetrope.

Goal progress comes from editable completion criteria with recorded evidence. Missing tracking is shown as unknown, not an invented percentage. Meaningful work uses existing short task descriptions or commentary; file operations remain concise. Questions and drafts persist locally until answered or deliberately dismissed. Generated Markdown documents are listed with the relevant work and can be opened in the user's VS Code MD Reader preview.

## User Stories

1. As an operator, I want `zoe` to show work across projects, so that I do not have to know a session ID first.
2. As an operator, I want active and waiting work emphasized, so that I can find what needs attention.
3. As an operator, I want one key to show all known history, so that I can revisit previous work quickly.
4. As an operator, I want history in the same graph, so that I do not have to learn another interface.
5. As an operator, I want to search project names, goals, session titles, agent names and document paths, so that I can find related work.
6. As an operator, I want archived sessions included when locally available, so that history does not omit finished work merely because it was archived.
7. As an operator, I want collapsed sessions to be inexpensive, so that a large history does not require parsing every transcript on every refresh.
8. As an operator, I want to expand a session into its actual agents, so that I can see delegation and parentage.
9. As an operator, I want stable selection and graph positions during refresh, so that the display does not move away while I inspect work.
10. As an operator, I want internal automatic review helpers grouped under their owning work, so that they do not look like unrelated user projects.
11. As an operator, I want existing replay controls preserved, so that I can inspect how a selected session unfolded.
12. As an operator, I want replay navigation separated from live actions, so that browsing the past does not send feedback to an unintended current run.
13. As an operator, I want the user's goal to remain visible, so that activity is assessed against the requested outcome.
14. As an operator, I want editable completion criteria, so that progress reflects the actual scope.
15. As an operator, I want evidence attached to verified criteria, so that I can inspect why progress advanced.
16. As an operator, I want progress independent of token usage, elapsed time and finished-agent counts, so that the bar means something useful.
17. As an operator, I want missing criteria shown honestly, so that an untracked goal is not mistaken for zero progress or completion.
18. As an operator, I want scope revisions preserved, so that discovered work does not silently rewrite earlier completion claims.
19. As an operator, I want testing and final checks included in completion, so that implementation alone is not presented as done.
20. As an operator, I want a persistent composer for the selected goal or agent, so that I can give feedback without switching applications.
21. As an operator, I want to choose between immediate guidance, later input and stopping work, so that these different actions are not confused.
22. As an operator, I want the recipient and delivery state visible, so that saved, queued and accepted input are not mislabeled.
23. As an operator, I want child-directed feedback to reach that child, so that the parent and siblings are not unintentionally redirected.
24. As an operator, I want unsupported controls to be explained, so that the display does not pretend to control a log-only session.
25. As an operator, I want native Codex access to continue working, so that Zoetrope adds another control surface without replacing it.
26. As an operator, I want work to continue by default, so that monitoring does not add mandatory interruptions.
27. As an operator, I want optional milestone checkpoints, so that I can choose closer supervision for a particular goal.
28. As an operator, I want short descriptions for meaningful work, so that I can understand its purpose quickly.
29. As an operator, I want routine file actions kept concise, so that trivial actions do not dominate the graph.
30. As an operator, I want Bash commands inspectable with existing stated purpose when available, so that I can understand what ran without paying for repeated summaries.
31. As an operator, I want unanswered questions to remain available, so that I can answer when I am ready.
32. As an operator, I want partial answers preserved across navigation and restart, so that I do not lose work.
33. As an operator, I want to answer or deliberately dismiss questions myself, so that default selections are never silently submitted.
34. As an operator, I want expired native questions distinguished from still-pending ones, so that the answer is not falsely reported as delivered.
35. As an operator, I want documents created or changed by agents linked to their work, so that I can read the actual result.
36. As an operator, I want to open a selected Markdown document in VS Code with MD Reader, so that it uses my existing reading setup.
37. As an operator, I want unavailable documents reported clearly, so that deleted files do not break the console.
38. As an operator, I want passive rendering and discovery to use no model calls, so that watching agents does not itself spend model tokens.
39. As an operator, I want clear keyboard help and usable narrow-terminal behavior, so that the console remains terminal-first.
40. As an operator, I want the old file/ID/inspect commands to remain usable, so that existing workflows and integrations still work.

## Implementation Decisions

- Introduce a fleet console module above the existing single-session application. Preserve the existing provider/fact/replay module contracts, and keep the portable core usable without native runtime features.
- Reuse provider discovery. Add a local incremental inventory with stable provider/session/agent identities, archived-session discovery, lazy expansion, and change-based refresh. Historical entries do not become active merely because they are loaded into the graph.
- Use `H` for the one-key current/history switch, `/` for search and visible keyboard hints. Search initially covers indexed names, goals, current descriptions and artifact paths. Full-text indexing of every tool output is outside this spec.
- Keep selection pinned through refresh and session expansion. Group automatic approval/review helpers under their real parent when known; provide a way to reveal them without making them independent top-level goals.
- Preserve single-session replay and file/ID targeting. The default no-target launch becomes the fleet console. Old project-follow behavior remains explicitly selectable.
- Give a goal a durable completion contract: stable outcome identifiers, labels, optional contribution weights, status, evidence references, provenance and scope revision. Equal contribution is the default. An outcome earns verified credit only with recorded evidence or explicit user attestation.
- Derive verified-outcome count and optional weighted completion locally. Tokens, time and playback position stay separate. Native plan steps can be imported as draft criteria; a plan status alone is not relabeled as verified evidence. When no contract exists, show “Progress not tracked” and provide an explicit tracking action.
- Permit criteria/evidence updates through a public local command interface so Codex and the operator can maintain the same record. Provide an explicit way to ask the selected agent to propose/update criteria; do not silently inject instructions into every agent or launch a second summarizer.
- Use Codex's existing shared local runtime where compatible. Negotiate the installed interface, read real status and active-turn identity, and send guidance only to the exact selected thread and expected active turn. Do not start another run to simulate attachment.
- Keep the live control adapter separate from transcript replay. Existing native Codex access and access policies remain authoritative. Do not overwrite model, approval, sandbox or instruction settings while joining a run.
- Treat co-client approval/question behavior as a compatibility requirement to verify on disposable sessions before claiming full coexistence. Failure to establish a supported action produces an unavailable action with a clear reason, not a hidden fallback.
- Never silently reroute a selected child's feedback or stop request to its parent. Let the user explicitly select the coordinator when direct control is unavailable.
- Persist feedback drafts and outbox status. Distinguish local draft, queued, accepted by runtime, rejected and disconnected/unknown delivery. Acknowledgment of acceptance does not prove that the requested change was completed.
- Keep queueing distinct from guidance and interruption. Preserve pending queue items across restart; do not automatically replay uncertain deliveries or send to a newer turn after a stale-turn rejection.
- Make continuous work the default. Optional milestone checkpoints are per-goal and disabled by default. Do not change the native runtime's existing permission rules.
- Capture supported structured questions and local drafts with stable source identity. Observe answers resolved elsewhere when the runtime exposes them. No automatic timeout or default submission is introduced by Zoetrope.
- If a native question has expired, retain its text and draft. Explain that the original request is no longer pending and offer an explicit send-as-feedback action; do not claim the original request was answered.
- Reuse task names, existing commentary and deterministic file-action labels. Routine reads/writes remain concise. Do not fabricate why a command ran, surface protected reasoning, or call a second model to relabel every event.
- Preserve structured artifact paths and producer identity when a supported transcript records document creation/change. The operator explicitly opens documents; merely reading a path in a transcript does not launch an application.
- Integrate the installed MD Reader extension with a small local VS Code URI bridge. Open the selected file, then invoke MD Reader's contributed full-render command. Without a verified render acknowledgment, say that preview was requested. Fall back to opening the file in VS Code when preview integration is unavailable.
- Store Zoetrope-owned state locally with versioned records and atomic/concurrent update handling. Native Codex transcripts remain read-only. New local IPC is an explicit extension of the old read-only product contract, while internet communication and telemetry remain absent from the viewer.
- Support Codex control on the user's macOS environment first. Preserve Claude transcript viewing and platform-independent replay; do not claim Claude or Windows live-control support without an implementation and verification.
- Follow the requested integration-branch workflow, with dependency-aware tickets, separate implementer worktrees, TDD at the agreed seams, merger review and final code review. Keep human answers and source research available to implementers.

## Testing Decisions

These two public testing seams were approved before writing new tests:

1. **Fleet console interface:** supply real-format session fixtures, durable state and external runtime messages, interact through public keyboard/mouse commands, and observe the rendered console and external actions. This catches cross-project discovery/history/search, nested identities, goal-progress rules, readable work, persistent drafts/questions, exact feedback target and delivery state, replay isolation, and document-open requests. It does not establish real terminal font quality or compatibility with every vendor release.
2. **Executable and launcher interface:** run the actual command with an isolated local session/state directory and a local Codex protocol peer. Verify arguments, default fleet launch, legacy commands, live protocol exchange, persistence across process restart, and the editor bridge's public URI/launch behavior. This catches wiring and lifecycle failures; it does not test model intelligence or every VS Code configuration.

Tests assert externally visible behavior through these interfaces, rather than private methods, internal collaborator calls, or snapshots mechanically copied from the implementation. A local protocol peer is an adapter for the external runtime seam, not a mock of internal modules. Use independent fixtures and explicit expected outcomes.

Prior art includes the existing public input-handler tests, ratatui buffer rendering, provider fixture conformance and executable argument tests. Implement one vertical slice at a time: demonstrate a meaningful failing test, make that behavior pass, then take the next slice. Do not write an entire suite for imagined future internals before implementation.

Run the existing repository checks appropriate to the final changes: formatting, warnings-as-errors clippy, locked tests, portable-core check, native release build, and browser/wasm compatibility checks when shared code changes. Validate local runtime coexistence on explicitly disposable sessions, and verify the installed binary and document preview on this Mac.

Baseline before changes: 207 library tests and 9 binary tests passed with the locked dependencies; no product code was modified. Deterministic console/integration tests verify the viewer and control behavior, not the correctness of an agent's general reasoning. Live compatibility checks separately verify real Codex control and editor behavior without claiming to evaluate model intelligence.

## Out of Scope

- Replacing Codex's agent harness, changing models, or adding automatic model-based supervisors.
- An exact prediction of remaining time or effort for arbitrary goals.
- Claiming control of private/unreachable runtimes, silently duplicating sessions, or routing child feedback to another agent.
- Automatically answering native approvals, changing sandbox policy, or taking ownership of approvals from another client without verified runtime support.
- Full transcript/tool-output search, cloud/remote fleet management, a new GUI application, and a new browser frontend for the fleet console.
- Publishing the VS Code bridge to a marketplace or changing the user's default Markdown renderer.
- Modifying or deleting native session history.

## Further Notes

The answered questionnaire remains the product source of truth. The installed fork is the source checkout used for development, and the `zoe` command points to its release build.

Read-only investigation verified a working second-client connection to the current Codex shared runtime and real thread/goal/turn metadata. Mutating controls and multi-client approval coexistence still need disposable-session validation. The actual installed Markdown plugin is MD Reader 0.0.8; its public full-render command is known, but the external bridge has not been installed or exercised.

The approved tracker is the fork's own GitHub Issues, with a `ready-for-agent` label and one integration PR closing the approved spec and tickets. The user approved enabling Issues, publishing this spec and its tickets, and the integration PR workflow.
