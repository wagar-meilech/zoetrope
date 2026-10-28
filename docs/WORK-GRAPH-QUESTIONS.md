# Visual work graph and workflow tracking — design questions

Status: Q1–Q5 answered; product direction approved by “Answered, proceed.” The concrete [specification](WORK-GRAPH-SPEC.md) and [six-ticket plan](WORK-GRAPH-PLAN.md) carry these answers forward. The task breakdown is the remaining pipeline review checkpoint.

## Already settled

- Build a real visual work/dependency graph. `w` must open it, rather than only focus a text list.
- Package tracking as a reusable skill that fits the Matt Pocock workflow.
- Aim for one tracking request followed by ordinary project documentation/ticket maintenance.
- Improve color coding, readability and navigation.
- Keep Rust, bounded local work and low token cost. An additional model remains out of scope unless separately enabled and designed.
- Preserve deliberate steering, exact recipients, native access and replay. Reported completion remains distinct from verified outcomes.
- Keep the existing public console and executable/external testing seams and vertical TDD.

## What investigation established

- The live goal already has stable work IDs and declared dependencies; these are enough to draw its graph without a model.
- Its process inherited `NO_COLOR=1`, which suppresses existing color output. The console also has little status-specific coloring. Both need attention.
- Matt’s skills share the project’s tracker instructions; they do not expose a reporting-hook API. A project convention is the practical integration point.
- Project setup and native session identity are separate: future goals must not inherit an earlier coordinator’s session ID.
- Some reports use logical child names. Public session metadata can resolve these to native thread IDs; unresolved or ambiguous owners must remain explicit.

[Exploration findings](research/work-graph-tracking-exploration.md) record the supporting facts.

## Round 1 — Current decision frontier

❓ **Q1** - **Navigation and visual flow**: Should Fleet and Work graph be distinct views, with `w` or Enter on a selected goal opening its Work graph, a visible Fleet → Project → Goal breadcrumb/back action, and clearly labeled Agents and Replay views?

➡️ **Recommendation: yes.** Fleet arrows select goals without changing views. Opening a goal shows tasks and their declared dependency edges, with readable status/owner cues. Keep purpose, changes, next steps, blockers and steering in the selected task’s detail panel. Preserve view/selection/zoom when going back. Keep an overview plus focused detail on small terminals, rather than shrinking the work graph to five rows.

**My answer / changes:**
Yeah, sounds good
<!-- Write here. -->

---

❓ **Q2** - **What keeps tracking current**: Should the existing project task documents/tickets become the authoritative work records, with Zoetrope importing their structured changes, rather than asking the coordinator to maintain a second plan through separate reporting commands?

➡️ **Recommendation: yes.** The skill discovers the project’s convention and adds the smallest missing status/owner/dependency fields to that workflow. Agents maintain their usual artifacts. Zoetrope reads those changes into a local visualization cache. Compact explicit reports remain available for context/evidence the normal artifacts do not carry. A skill alone cannot subscribe to file changes; a bounded Rust adapter/helper supplies the automatic part. Source formats and remote synchronization details follow this decision.

**My answer / changes:**
If it exists - yes, and then the coordinator can just provide that, but we don't want this to be a dependency (so it should have a fallback if the matt potock workflow is not being used)
<!-- Write here. -->

---

❓ **Q3** - **What “request once” means**: Should activation be a persistent opt-in for the selected project, covering later sessions as well as the current goal?

➡️ **Recommendation: yes.** Install a normally discoverable tracking skill for this user, and let project activation add a small binding/configuration plus a conditional pointer in that project’s existing instructions/tracker convention. Keep native session IDs in local state. New goals bind to their own coordinator; unrelated projects remain unaffected. Provide a way to disable project tracking. Leave upstream Matt Pocock skill files intact.

**My answer / changes:**

<!-- Write here. -->
Yes, the idea is to minimize token usage, though - so the less requests we have to send the coordinators way, the better
---

❓ **Q4** - **Color and readability**: Should we reuse the original app’s visual language across the fleet, work graph, details and actions, with consistent status colors, selection emphasis and an always-visible legend?

➡️ **Recommendation: yes.** Combine color with text/glyphs, distinguish active, blocked, planned, reported-done and verified states, and keep labels useful when zoomed out. Respect intentional `NO_COLOR` in automatic mode; provide an explicit color override and enable it when relaunching our attached demo. Keep terminal configuration changes scoped to Zoetrope.

**My answer / changes:**
Yes, and maybe we should add more - making the interface more well defined, nice visual, intuitive color coded elements and graphs instead of walls of text
<!-- Write here. -->

---

❓ **Q5** - **Progress that matches the graph**: Should the header show reported work completion alongside verified outcomes, so a project with several finished tasks does not communicate only “0% verified”?

➡️ **Recommendation: yes.** Show separate labeled counts, for example “5/11 work items reported done · 1 active” and “0/5 outcomes verified,” with source/update information. A task count is not an estimate of remaining effort or delivery time. Existing documents or ticket closure alone do not automatically verify outcomes.

**My answer / changes:**
Yes
<!-- Write here. -->

## What follows

Your answers settle the product direction, including the non-Matt fallback and stronger visual hierarchy. The mechanism follows the existing local-monitoring constraint: supported local structured artifacts are imported; normal agent tracker operations remain separate and source/freshness are labeled. The specification defines precedence, errors and per-goal binding. Review the concrete mechanism and six-ticket breakdown together in the linked plan; then continue publication, vertical TDD, merger integration, independent review, installation and live demonstration.

The [grilling skill](/home/dust_60/.agents/skills/grilling/SKILL.md) requires: “Do not act on it until the user confirms you have reached a shared understanding.” No application, installed-skill, live-session or external-project changes are made during this question round.
