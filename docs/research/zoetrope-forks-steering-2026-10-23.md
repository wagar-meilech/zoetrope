# Zoetrope forks and steering audit

Snapshot: 23 October 2026. Read-only research; no product installed, built, or exercised. Scope: public metadata across the fork network, current upstream open PRs/issues, and bounded inspection of the most popular and most relevant named/active forks. This is not an audit of every branch in every fork.

## Result

There is one materially relevant control fork: **[bloodf/herdr-sheppard](https://github.com/bloodf/herdr-sheppard)**. It adds a terminal popup and optional browser fleet manager to Zoetrope's inherited graph/replay engine. However, it explicitly rejects prompts to busy or blocked agents. Its active-agent control is confirmed Ctrl+C interruption, so it does **not** implement the requested non-destructive mid-run steering.

No prominent established steering fork emerged from the metadata audit. The largest fork has three stars; its default branch is behind upstream with no unique commits. The relevant Sheppard fork has zero stars. Small adoption is not a judgment of code quality, but it rules out treating it as a widely validated alternative.

## Fork metadata

The [public GitHub forks API](https://api.github.com/repos/furkankly/zoetrope/forks?per_page=100&sort=stargazers) returned **78 forks**, with `per_page=100`, so this response covers the visible public fork list in one page. Browser-rendered API JSON was used because shell networking and web-tool access to this endpoint failed. GitHub's [active-forks view](https://github.com/furkankly/zoetrope/forks?include=active&page=1&period=2y&sort_by=stargazer_counts) was also read. Dates below are API push timestamps; pushing does not prove a unique feature was added.

| Fork | Stars | Last push (UTC) | Findings |
| --- | ---: | --- | --- |
| [Shubhamsaboo/zoetrope](https://github.com/Shubhamsaboo/zoetrope) | 3 | 2026-08-25 | Most starred. Default branch shows 12 commits behind upstream, no ahead commits; README still describes the earlier read-only Claude viewer. |
| [mgrillo75/zoetrope](https://github.com/mgrillo75/zoetrope) | 1 | 2026-08-22 | Metadata only; no popularity signal beyond one star. |
| [hah23255/zoetrope--PhysicsAI](https://github.com/hah23255/zoetrope--PhysicsAI) | 1 | 2026-09-15 | Metadata only; same agent-graph description. |
| [Virtual0ps/zoetrope](https://github.com/Virtual0ps/zoetrope) | 1 | 2026-09-01 | Metadata only; same earlier viewer description. |
| [bloodf/herdr-sheppard](https://github.com/bloodf/herdr-sheppard) | 0 | 2026-10-11 | Substantial manager/control addition; discussed below. |
| [Ispagiytiy/zoetrope-codex](https://github.com/Ispagiytiy/zoetrope-codex) | 0 | 2026-08-25 | Default branch 24 commits ahead and 12 behind upstream. README adds Codex ingestion, source selection, live child-file discovery and privacy details, but explicitly preserves read-only behavior. |
| [PremModhaOfficial/zoetrope](https://github.com/PremModhaOfficial/zoetrope) | 0 | 2026-10-23 | Recently active, but its default branch is up to date with upstream and retains the same read-only README. Other branches were not exhaustively inspected. |

All other returned forks had zero stars. Other recent activity included Trinityy2 (23 October), philmcneely (21 October), jsheely and xiaofeilovecoding (20 October); recent push times alone did not establish goal/steering features. The upstream PR list identifies jsheely's contribution as Windows Herdr support and xiaofeilovecoding's as Claude configuration-directory discovery.

## Sheppard: useful control code, incomplete match

The [root README](https://github.com/bloodf/herdr-sheppard) distinguishes three surfaces: a popup/fleet dashboard operating native Herdr agents; the inherited Claude/Codex transcript graph; and optional OMP task/phase/parent metadata. It requires Herdr 0.9.3 and Python 3; the manager is not a standalone Codex app-server client. The package states that it makes no AI calls.

The [manager controls](https://github.com/bloodf/herdr-sheppard/blob/main/manager/README.md) are:

- Focus an agent and inspect recent output.
- Send a prompt only when native state is idle/done and `interactive_ready=true`.
- Reject prompts for busy or blocked agents.
- Interrupt a working/blocked agent through Ctrl+C after confirmation.
- Display assignments and native state; optional metadata adds current task, phase, owner/parent and model.

It does not document a whole-goal percentage, semantic command explanations, or sending steering text directly to an actively running graph node. Its own documentation says native "done" is a settled unseen completion state, not acceptance of the task or validation of the phase. Graph actions are separate from the manager, and the graph does not parse OMP/Pi transcripts. These boundaries are why this is a useful reference for adaptation, rather than a ready match.

The fork's main history visibly contains the new [scoped worker-manager implementation](https://github.com/bloodf/herdr-sheppard/commit/97248957c5034280703cc9ae031f04ae4b427d9f) plus registration/installation documentation changes. Its metadata and release surface show zero stars and no independently published package release; runtime behavior has not been tested in this research.

## Upstream status

The [upstream README](https://github.com/furkankly/zoetrope) explicitly defines Zoetrope as a read-only transcript viewer. Its pause and timeline controls change playback, not the running agent. The architecture separates the portable graph/model from native file I/O and the browser frontend. A control transport would therefore be an additional integration, rather than an existing feature hidden behind a shortcut. [Design document](https://github.com/furkankly/zoetrope/blob/main/docs/DESIGN.md)

The current [open PR list](https://github.com/furkankly/zoetrope/pulls) contains 13 items: provider additions, dependency maintenance, localization, themes, Windows integration and one display-summary improvement. None of these titles/body surfaces establishes active steering or whole-goal progress. The two [open issues](https://github.com/furkankly/zoetrope/issues) concern Claude configuration-directory discovery and Hermes support.

One small relevant change is [PR #31](https://github.com/furkankly/zoetrope/pull/31): it displays the skill name instead of a bare Skill label. The patch extracts a known field; it does not call an LLM. It applies to Claude skill records, and the author explicitly excludes Codex and card/chip labels. This is evidence that friendlier labels can sometimes be produced by ordinary parsing, without token overhead; it is not the user's requested explanation layer.

## Implication for a possible adaptation

Start from current upstream if Codex is the main runtime; its graph and provider separation are useful. Borrow ideas from Sheppard for selected-agent controls, target identity, delivery receipts and honest state reporting, while replacing its idle-only prompt path with a supported Codex steering transport. Root-agent research should verify the shared app-server/socket approach separately. Add goal/milestone state and readable activity summaries as explicit data rather than treating the playback slider or tool count as goal progress.

UI labels derived from known events cost no model tokens. An authentic explanation of why a command was chosen requires information already present in the agent's commentary/tool metadata, or a small additional authoring instruction/model output. It should not be fabricated from the command text alone. This paragraph is an engineering inference, not an existing Zoetrope capability.
