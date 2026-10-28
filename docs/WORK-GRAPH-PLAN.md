# Work graph implementation plan

Product decisions: [answered questions](WORK-GRAPH-QUESTIONS.md).
Concrete behavior: [specification](WORK-GRAPH-SPEC.md).
Status: approved by “Approved, proceed.” All six tickets, blocking edges, mechanism and existing public test seams are authorized. Implementation follows the frontier below.

## Mechanism carried forward

- Existing reports immediately become a Work graph; this does not wait for the new tracking skill.
- Existing structured local task documents/tickets supply ongoing status when available. The Rust importer reads changed, declared sources; the working agent keeps using its normal documents and remote ticket tools. Source/freshness labels distinguish local reports from remote facts.
- Without Matt's workflow or suitable files, native plans/compact reporting still work. The skill can establish one minimal task document, avoiding a mandatory Matt setup or a second plan where one already exists.
- One project opt-in installs a conditional convention. Each goal retains its own exact native binding. Discovery never picks a session from a project name or its recency.
- Source-managed fields stay attributable; supplemental reports fill gaps. Invalid source edits retain the last good view and show a warning. No source status automatically verifies an outcome.
- Passive monitoring stays local under the existing no-internet constraint. No additional model is introduced.

## Live task frontier

This table is the working-status source; numbered tickets below define acceptance.
Statuses are coordinator reports, distinct from outcome verification and issue closure.
The [final validation](WORK-GRAPH-VALIDATION.md) records the completed integration;
issues remain open until PR #8 merges.

| ID | Task | Status | Owner | Depends on | Next |
| --- | --- | --- | --- | --- | --- |
| WG-1 | Navigate a real work/dependency graph | done | /root/wg1_canvas | None | Integrated. |
| WG-2 | Steer the actual work owner | done | /root/wg2_owner | None | Integrated. |
| WG-3 | Follow structured project artifacts locally | done | /root/wg3_sources | None | Integrated. |
| WG-4 | Activate the reusable tracking skill | done | /root/wg4_skill | WG-3 | Integrated. |
| WG-5 | Make the graph and controls visually legible | done | /root/wg5_visual | WG-1 | Integrated. |
| WG-6 | Integrate, review and update the live installation | done | main | WG-2, WG-4, WG-5 | Checks/reviews passed; app/skill installed; PR prepared for review. |

## Published issues

The [spec is #15](https://github.com/wagar-meilech/zoetrope/issues/15).
All six tickets are native sub-issues; blocking edges are also native GitHub dependencies.

| Ticket | Issue | Blocked by |
| --- | --- | --- |
| WG-1 | [#16](https://github.com/wagar-meilech/zoetrope/issues/16) | None |
| WG-2 | [#17](https://github.com/wagar-meilech/zoetrope/issues/17) | None |
| WG-3 | [#18](https://github.com/wagar-meilech/zoetrope/issues/18) | None |
| WG-4 | [#19](https://github.com/wagar-meilech/zoetrope/issues/19) | #18 |
| WG-5 | [#20](https://github.com/wagar-meilech/zoetrope/issues/20) | #16 |
| WG-6 | [#21](https://github.com/wagar-meilech/zoetrope/issues/21) | #17, #19, #20 |

## Approved vertical tickets

1. **WG-1 — Navigate a real work/dependency graph**
   - **Blocked by:** None.
   - **Delivers:** Existing work reports/native plans draw as selectable task nodes and declared dependency edges. `w`/Enter opens Work; Fleet/back/Agents/Replay routes are explicit. Selection/camera survive refresh, disconnected/cyclic/missing references remain inspectable, and selected details prioritize useful context. Includes public graph/input regression tests.

2. **WG-2 — Steer the actual work owner**
   - **Blocked by:** None.
   - **Delivers:** Existing logical owner paths resolve through scoped public native metadata. Inspection and guidance/queue/stop use the exact matching thread; missing/ambiguous mappings remain unavailable, and open drafts stay pinned. Works before and after the new graph lands; includes public runtime-peer coverage.

3. **WG-3 — Follow structured project artifacts locally**
   - **Blocked by:** None.
   - **Delivers:** Small idempotent tracking commands establish project opt-in and exact goal/source bindings, show status, refresh and disable. Local task tables/ticket files import through background IO into the existing work view without per-change model calls. Tests cover source edits, identity/isolation, precedence, stale/error recovery, restart, bounded IO/output and unchanged-data behavior. Existing report/plan fallback stays usable.

4. **WG-4 — Activate the reusable tracking skill**
   - **Blocked by:** WG-3.
   - **Delivers:** A packaged, validated, normally discoverable skill fits project tracker instructions and the Matt pipeline, preserves upstream skills, and supplies a non-Matt fallback. Enable tracking prepares a short exact-target activation request. One opt-in plus normal document maintenance keeps work live. Installation/update ships the skill with the app. Includes disposable-project behavioral verification.

5. **WG-5 — Make the graph and controls visually legible**
   - **Blocked by:** WG-1.
   - **Delivers:** Consistent status colors/glyphs, labeled views/breadcrumb/legend, compact details, separate reported/verified progress, semantic zoom and a useful narrow-terminal layout. Explicit auto/always/never color controls handle inherited NO_COLOR. Includes style, navigation, real-terminal/color and small-screen verification.

6. **WG-6 — Integrate, review and update the live installation**
   - **Blocked by:** WG-2, WG-4, WG-5 (WG-1 and WG-3 are transitive prerequisites).
   - **Delivers:** Combined native/portable/browser checks, independent Standards and Spec reviews and fixes, representative graph/source/resource evidence, updated usage/handoff, installed app/skill and a read-only live demonstration. Publish spec/tickets and update the existing PR; issues close through that PR when merged. Do not merge into main.

## Public test seams

Use the already approved console input/render + runtime-message seam and executable/external-file/process seam. No tests against private graph/parser helpers. Actual skill exercises use disposable projects; the ongoing bot project remains an observation target unless the user explicitly activates setup there.

## Breakdown approval

The `to-tickets` workflow asks for review of ticket granularity and real blocking edges before publication/implementation.

**Does this six-ticket granularity and dependency graph look right? Should any tickets be merged or split, or blockers changed?**

“Yes” approves this breakdown, the concrete mechanism/spec and use of the existing public seams. Otherwise write changes below.

**My answer / changes:**
Yes, looks good

<!-- Write here. -->

The user approved the breakdown and its dependencies. Continue issue publication, isolated TDD worktrees, merger integration, independent review, installation and live observation without requesting the same approval again.
