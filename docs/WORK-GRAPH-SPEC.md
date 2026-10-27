# Visual work graph and persistent workflow tracking

Status: approved through Q1–Q5, “Answered, proceed,” and final “Approved, proceed.” The [six-ticket plan](WORK-GRAPH-PLAN.md), mechanism and existing public testing seams are approved. Continues the existing integration. Published as [issue #15](https://github.com/wagar-meilech/zoetrope/issues/15).

## Problem Statement

An operator enables tracking and receives meaningful work reports, but `w` leaves the canvas showing agents. Task dependencies remain text, the visual hierarchy is unclear, and detailed commands overwhelm the useful context. A verified-outcome percentage alone can read as no progress despite finished work. Color output can disappear through an inherited terminal setting, and the console has little status styling even when color works.

Reporting also asks coordinators to maintain parallel state through repeated JSON commands. Existing project specs, ticket graphs and tracker conventions should do more of that work. Projects outside the Matt Pocock workflow need an equally usable fallback.

## Solution

Make Fleet and Work graph distinct, clearly labeled views. Open a goal as a graph of meaningful tasks and declared dependencies, with compact status/owner cues, separate reported-work and verified-outcome counts, focused contextual details and exact-recipient steering. Apply consistent colors and non-color cues throughout the terminal flow.

Provide a normally discoverable tracking skill and project opt-in. Use existing structured local task artifacts where available, importing changes deterministically in Rust. Reuse the project's tracker conventions without rewriting upstream skills. Retain native plans and compact reports, and offer a minimal task document when no suitable convention exists. One setup request establishes the convention; routine work then maintains its usual artifacts.

## User Stories

1. As an operator, I want `w` to open actual work nodes, so that a goal's approach is visible.
2. As an operator, I want dependency edges, so that parallel and gated work are distinguishable.
3. As an operator, I want unknown dependencies labeled, so that order is not invented.
4. As an operator, I want disconnected tasks and unresolved references visible, so that incomplete plans remain usable.
5. As an operator, I want cyclic declarations explained, so that malformed plans do not freeze the app.
6. As an operator, I want parent grouping distinguished from prerequisites, so that hierarchy does not imply scheduling.
7. As an operator, I want Fleet, Work, Agents and Replay visibly separated, so that navigation is predictable.
8. As an operator, I want breadcrumb/back navigation, so that I retain my place across views.
9. As an operator, I want keyboard and mouse selection, so that tasks are easy to inspect.
10. As an operator, I want selection and camera stability during refresh, so that incoming reports do not disrupt inspection.
11. As an operator, I want an effective small-terminal layout, so that the graph remains useful on my device.
12. As an operator, I want semantic zoom and an overview, so that larger graphs remain navigable.
13. As an operator, I want compact purpose, next action and blocker details, so that routine tool output does not dominate.
14. As an operator, I want clear status colors plus labels/glyphs, so that status is legible in color and monochrome.
15. As an operator, I want an explicit color override, so that an inherited NO_COLOR setting does not make color restoration obscure.
16. As an operator, I want reported-work counts beside verified outcomes, so that these two kinds of progress remain understandable.
17. As an operator, I want exact task-owner steering, so that logical agent names do not break targeting.
18. As an operator, I want ambiguous or unavailable ownership explained, so that feedback is never silently rerouted.
19. As an operator, I want open drafts to retain their recipient, so that updates cannot change whom I am addressing.
20. As an operator, I want one project tracking setup, so that I do not repeat instructions at each ticket.
21. As an operator, I want later goals bound to their own native identities, so that project opt-in does not reuse an old session.
22. As an operator, I want to disable project tracking, so that opt-in is reversible.
23. As an operator, I want existing task documents reused, so that there is one maintained plan.
24. As an operator, I want source and freshness information, so that cached local progress is not mistaken for current remote issue state.
25. As an operator, I want missing or malformed sources explained without losing the last good view, so that document editing is safe.
26. As an agent, I want stable IDs and structured source fields imported automatically, so that normal document maintenance replaces repeated state JSON.
27. As an agent, I want compact supplemental reporting, so that context absent from normal artifacts can still be recorded cheaply.
28. As an agent, I want Matt Pocock conventions recognized, so that tracking fits the existing spec/ticket/implementation workflow.
29. As an agent, I want a fallback without Matt's skills, so that tracking does not depend on that workflow.
30. As an operator, I want native recordings, existing state and browser replay preserved, so that the new view fits the app.
31. As an operator, I want bounded local monitoring without another model, so that visualization remains resource-light.
32. As an operator, I want the installed app and tracking skill updated together, so that new tracking requests use the current interfaces.

## Implementation Decisions

### Work graph and navigation

- Retain Fleet for selecting projects/goals. `w` or Enter on a goal opens a dedicated Work graph. Arrows in Fleet select without drilling in. A visible breadcrumb/back action and Esc restore the preceding view/camera/selection. Agents and Replay remain explicit routes; existing replay Enter/Space behavior remains intact.
- Work nodes use stable explicit IDs or revision-qualified native-plan identity. Draw only declared prerequisites. Parent links are grouping, and native-plan sequence does not invent dependencies. Unknown, empty, unresolved and cyclic dependencies remain distinct.
- Reuse the existing graph engine and theme patterns. Place isolated tasks explicitly. Bound nodes, edges, layout work and caches; show omissions. Relayout for topology/meaningful geometry changes, preserve placement on ordinary status updates, and keep IO off the drawing thread.
- Use compact node cards and semantic zoom. Keep a substantial graph area on narrow terminals, with a compact inspector and deliberate expansion/scrolling. Default details prioritize status, owner, purpose, next action and blocker; full command/public-message bodies stay behind Commands.
- Keep work selection, actual owner, capability, outcome links and delivery states connected. Preserve stale scope indicators, expired native selection, feedback pinning and explicit coordinator recovery.

### Progress and colors

- Show labeled reported-work counts, including active/blocked/unknown/cancelled where applicable, beside the existing evidence-backed outcome count/percentage. Task counts are not an effort estimate or ETA, and cancelled work is not reported done.
- Apply consistent theme/status cues to graph, selection, panels, actions and progress. Text/glyphs carry meaning when color is absent. Preserve a visible legend and legible overview selection.
- Offer auto/always/never color modes. Auto respects NO_COLOR; an explicit override is process-scoped. Verify actual ANSI behavior through a disposable terminal, including tmux-compatible use, rather than relying on stripped captures.

### Exact ownership

- Resolve collaboration-path owners through existing public provider metadata, scoped to the exact provider/root session and native ancestry where needed. Require an unambiguous mapping; retain the reported alias/provenance and explain missing/ambiguous matches.
- Resolve before selecting a live recipient and keep a draft pinned to that exact native recipient. Do not guess by project, title, recency or similar labels, and do not redirect to the coordinator.

### Persistent project tracking and source import

- Keep project opt-in/convention separate from per-goal native binding. Shared project metadata contains portable source conventions; machine paths, exact session IDs, receipts and caches stay in local state. Multiple goals/worktrees keep distinct bindings.
- Provide a small tracking command interface for idempotent enable/bind, status, disable and explicit refresh. A tracking request supplies its exact native goal and installed executable. A future coordinator uses an exact identity exposed by its runtime; ambiguity leaves setup visibly incomplete rather than attaching by cwd.
- Recognize explicitly designated local Markdown task tables and individual ticket files with stable IDs, status, owner and declared dependencies. Select a table/metadata area rather than parsing every table or narrative example; ignore fenced examples. Duplicate IDs are invalid. Support the project's current aggregate-document style and local Matt ticket-file style. Reuse existing documents, adding only missing structured fields through the opted-in workflow. Unknown statuses/owners remain visibly unknown or attributed human/unassigned values; they do not create a steerable native agent.
- Import bounded, explicitly allowlisted source changes in the existing background local-IO flow. Validate resolved paths and bounded file/directory sets; do not execute content or crawl arbitrary symlinks. Retain source identity, source fields, timestamps/digests and last successful synchronization. Identical content is a no-op. Preserve stable work identity across ordinary edits. Namespace source identities within the exact goal binding; do not merge tasks by similar labels. Source changes apply only to their declared goal/source scope, not every goal in a project.
- Source-managed fields follow their declared source; explicit reports enrich fields the source does not manage. A present-but-empty source field clears its source value; schema absence does not resurrect a stale override. Retain per-field provenance and surface conflicts or required source edits rather than oscillating between two plans. A valid snapshot removing a task retires it from current work while preserving history and expiring its selection; an explicitly empty source is valid. Invalid, oversized or temporarily unavailable input instead retains last-good data with a stale/error explanation. Disabling tracking stops refresh without erasing goal evidence/history.
- Passive monitoring remains local: it does not poll GitHub or launch network commands. Existing agents can continue their normal tracker tools. For a remote-authoritative project, local artifacts/reports are an attributed working projection linked to the remote tickets, not a claim of freshly fetched remote state.
- Without suitable local artifacts, reuse native plans and existing compact reports. The skill can establish one minimal local task document for a project lacking a convention. Matt Pocock skills are optional; source absence does not prevent visualization or deliberate steering.
- Structured source status and checklists never automatically verify outcomes. Preserve the separate evidence interface and scope revisions. Source snapshots have an as-of timestamp; recorded Replay stays read-only and never silently overlays today's documents onto an earlier transcript time.

### Tracking skill and activation

- Package a normally discoverable skill with short conditional project guidance and source-specific references. Use the project tracker convention as the Matt integration seam; leave upstream skill definitions intact.
- Activation preserves existing project instructions and introduces only a managed opt-in pointer/configuration. Make repeated setup harmless, preview changes, and disable only managed tracking additions. Do not bind future goals to a historical coordinator.
- The explicit setup skill owns managed instruction/tracker-pointer edits; a passive source refresh or binding command never rewrites project instructions. Use exact runtime identity variables when available, validated against public metadata; missing/mismatched identity remains explicit. Ask the coordinator once to select/bind the current goal's sources and maintain its usual artifacts at meaningful transitions. Supported structured sources require no repeated per-item state JSON. Supplemental reports remain compact and optional where normal artifacts lack semantic fields.
- Distinguish explicit user activation from merely installing/discovering the skill. Provide a fallback path when the skill is unavailable; prepare an editable request using the installed executable and exact local identity.

## Testing Decisions

Reuse the two already approved public seams:

1. Console input/render plus external runtime messages: graph topology as displayed, navigation/back, keyboard/mouse selection, stability, narrow/overview behavior, colors, progress labels, exact owner, pinned drafts and replay isolation. Does not establish every terminal's appearance or that an agent follows instructions.
2. Executable and external connections/files: tracking activation/import/disable, ordinary source edits, source conflict/failure/restart behavior, compact output, native identity mapping, color modes and installer/skill handoff. Does not establish universal arbitrary-Markdown interpretation or runtime-version compatibility.

Use fixtures and actual public processes, one meaningful RED/GREEN slice at a time. Keep parser/layout implementation details out of tests. Style-buffer checks and actual terminal probes complement each other. Exercise the skill in a disposable project with realistic source artifacts, including a non-Matt fallback and resumed work; validate behavior rather than copied wording.

Run locked native checks, strict Clippy and formatting, the declared Rust minimum, portable/browser compatibility and relevant package/document checks. Measure representative graph/source refresh and live/idle resources. Perform live observation without automatically steering another active project or rewriting its instructions.

## Out of Scope

- Additional summarizing models, opaque reasoning/KV access, automatic network polling or telemetry.
- Replacing the agent harness or changing delegation/scheduling.
- Universal freeform document understanding, automatic source discovery by scanning the entire disk, or remote issue writes by the monitor.
- Verification inferred from reported done, checkboxes, issue closure, token counts or elapsed time.
- Rewriting upstream Matt skills, enabling unrelated projects, automatic native input or merge into main.

## Further Notes

The answered questionnaire is the product source of truth. [The implementation plan](WORK-GRAPH-PLAN.md) records the concrete task frontier. Continue on the current feature integration/PR and update the installed connected-environment app and user skill after verification. Existing explicit reports remain compatible and populate the new graph immediately.
