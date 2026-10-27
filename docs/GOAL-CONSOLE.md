# Using the goal console

Run `zoe` to see work across local projects. The native app groups projects,
goals and agents into an expanding graph. Selecting a goal shows its meaningful
work and evidence-backed progress. `H` switches Current / All history, `/`
searches, and `?` shows help. Selection stays with the same work during refresh.

To run the current source checkout, build with `cargo build --release --locked`
and launch `./target/release/zoe`. This semantic-context iteration was developed
and checked on Linux; it has not been installed as a global `zoe` command here.

Current emphasizes working agents and requests needing attention. Without a
compatible live connection, recent transcript writes are a fallback signal;
they do not prove that an agent is running or that its goal is finished. All
history includes locally available archived Codex sessions. Search covers
indexed names, goals, criteria, explicit work IDs/labels/context/owner IDs,
recorded plan text and document paths, rather than every tool output.

## Understanding the work

Fleet selects projects and goals. Press `w`, Enter, or **Work** on a goal to open
its task/dependency graph, then use arrows or `[` / `]` to choose a task. **Back**
or Esc restores the previous view, selection and camera. Declared prerequisites
are arrows into a task; parent grouping never invents a prerequisite. Isolated,
unresolved and cyclic work remains visible. Recorded native plans are available
immediately; explicit reports replace the same owner's native plan fallback.

The legend uses ○ blue planned, ● cyan active, ! red blocked, ✓ green reported
done, × gray cancelled and ? magenta unknown. A gold border and **› selected**
identify selection. `o` fits the overview, retaining status/selection cues at
small zoom; `r` restores the selected task at readable size. Reported counts and
verified outcomes are separate: a completed task earns no outcome credit by itself.

A selected task's owner becomes the recipient for ordinary `f` steering. Before
opening Work, arrows select goals in Fleet. Press `a` or **Agents** for agent
inspection; `x` / **Commands** shows the selected owner's full cached public
messages and shell command; `R` / **Replay** opens the saved activity graph.
Commands stay behind that explicit route. Enter/Space expands agents in Agents.

Work detail shows its owner, source, recorded time and reported purpose/change/
next action/blocker. Dependencies are shown only when declared: **not declared**
means unknown; **none declared** is an explicit empty list. Missing work links
are unresolved. Outcome links retain their goal revision and become visibly
stale when scope changes. Older plans or messages can have unknown timestamps.
Recorded activity is an attributed report, not proof of current runtime activity.

Stable explicit work IDs preserve selection through ordinary edits. Replacing a
native plan invalidates the old plan selection; choose current work with `w` or
explicitly select **Coordinator** with `C`. The work view and its detail are
bounded; omission notices and compact context paging expose larger records.
On narrow terminals, current owner/purpose/next action and control availability
appear first; PageUp/PageDown scrolls the remaining detail.

## Keyboard controls

| Key | Action |
| --- | --- |
| `H` | Switch Current / All history. |
| `/` | Search the graph. |
| Up / Down or `j` / `k` | Navigate the fleet, or choose work after `w` focuses it. |
| `w`, then arrows or `[` / `]` | Focus and select meaningful work. |
| `a` / **Agents** in Work/Agents views | Open agent inspection; `w` returns to work. |
| `x` / **Commands** in Work/Agents views | Read the selected owner's cached commands/public messages. |
| Enter or Space | Open a goal's Work graph; expand/collapse agents in Agents. |
| Left / Right, `+` / `-`, `o`, `r` | Pan, zoom, fit the overview, restore readable size. |
| `f` / **Steer** | Edit feedback for the selected recipient. |
| `C` / **Coordinator** | Explicitly select this goal's coordinator; retain recipient-specific drafts. |
| `T` / **Enable tracking** | Prepare an editable coordinator reporting instruction. |
| `Q` | Open persistent questions. |
| `x` in Questions | Dismiss the selected question locally. |
| `L` in Questions | Listen for current native questions on the selected live agent. |
| `c` | Read completion criteria and evidence. |
| `d` | View recorded documents. |
| `e` | Edit a question answer or the completion criteria. |
| F5 in Questions | Copy a closed request's answer into a feedback draft. |
| F2 / F3 / F4, or Ctrl-S / Ctrl-Q / Ctrl-X in Feedback | Send guidance / queue / stop. |
| `b` | Read saved deliveries and their status. |
| `I` | Reveal internal helpers under their owning work. |
| Esc / **Back** | Restore the previous view; save an open feedback/answer draft; cancel a criteria edit. |
| `R` / **Replay** | Open read-only session replay; `q` returns to the fleet. |
| PageUp / PageDown | Scroll selected detail. |
| `?` | Show context and keyboard help. |

Ordinary characters stay in the editor while you type; they do not trigger
graph shortcuts. Question choices are saved without submission. Use F2 to
submit deliberately after answering every prompt. In the answer editor, F2
saves the partial answer and keeps editing; Esc returns to Questions after
saving. Tab moves between
prompts and number keys choose an option in Questions.

## Terminal colors

`zoe --color auto` respects a nonempty inherited `NO_COLOR` and enables color on
terminals. `zoe --color always` restores color for this process even when the
parent shell or tmux sets `NO_COLOR=1`; `zoe --color never` disables it. The flag
is global and also works with `--project`, replay and subcommands. Glyphs, labels
and the selected marker remain useful in monochrome.

## Goal progress

Reported work shows done, active, blocked, planned, cancelled and unknown counts
for the displayed task projection. These counts are not an effort estimate or
ETA; cancelled work is excluded from done. Source and recorded-time labels stay
with selected details, including stale source warnings.

A goal's completion criteria define its verified-outcome progress. Each verified criterion needs
recorded evidence or an explicit user attestation. The display shows both the
verified count and the percentage. Criteria contribute equally unless weights
are supplied; for example, verifying one criterion with weight 3 out of weights
3 and 1 gives **1 of 2 verified · 75%**.

**Progress not tracked** means there is no completion contract. It does not mean
0% or 100%. Native plan steps can be adopted as draft criteria; a plan marked
completed does not establish verified evidence. Include testing and final
checks when defining what done means.

Requesting criteria prepares a feedback draft for the selected recipient. Read
it and deliberately send it when ready. Zoetrope does not inject instructions
into every agent or run a summarizing model. Editing scope keeps earlier
revisions, and reopening a verified criterion removes its current credit while
retaining the earlier evidence.

In Criteria, `e` edits the goal title and one outcome per line, `v` records
evidence or your attestation, and `u` reopens the selected outcome. Use `[` / `]`
to select a recorded plan and `a` to adopt it as unverified criteria. `A`
prepares the criteria-request draft for the selected recipient.

Milestone checkpoints are off by default. Press `p` in Criteria to toggle the
local preference. Enabling it affects
the criteria-request draft; sending that guidance asks the agent to check in.
The preference alone does not pause Codex or change its permission rules.

## Enable tracking

Press `T` or click **Enable tracking** to prepare an editable reporting draft for
this goal's coordinator. It names the exact provider, session and local state
location, requests compact work updates only at meaningful transitions, and
preserves earlier feedback. When completion criteria are missing, it also asks
for concise editable `set_goal` outcomes including tests and final checks.

The draft is saved locally. Choose Send guidance or Queue deliberately to send
it; acceptance does not establish that reports will be maintained. Existing
plans and public messages remain useful without sending a tracking request.
For selected projects, the [agent reporting guide](AGENT-REPORTING.md) supplies a
small conditional project-instruction pointer and the reusable convention. The
operator chooses where to install it; Zoetrope does not edit project/global
instructions automatically. No helper model is used in this iteration.

## Steering and questions

Selected work exposes **Steer**, **Coordinator**, **Enable tracking**, **Work**,
**Agents**, **Commands** and **Replay** actions. Guidance, Queue and Stop show
separate availability before a send. A child that rejects direct input may
still accept an exact-turn Stop. Disconnected, unloaded and unavailable runtime
states are explained; **not loaded in Codex** does not prevent local history
inspection. A disabled send keeps the draft.

The composer belongs to the displayed recipient. A work item's owner supplies
that recipient when feedback opens; subsequent work reassignment does not
retarget an open editor. Drafts and partial answers remain available across
navigation and restart. Check the recipient before sending.
Closing a dirty composer waits for its local save to succeed. A storage error
keeps the text open for retry. An insertion exceeding the 65,536-byte editor
limit is rejected in full with a visible explanation; the earlier text stays
intact.

| Action | Meaning |
| --- | --- |
| Save a draft | Keep the text locally; send nothing. |
| Send guidance | Send to the selected live thread and its inspected active turn, or explicitly submit new input to a loaded idle thread. |
| Queue | Submit input to Codex's native queue. An idle thread may start immediately. |
| Stop | Request interruption of the selected thread's inspected turn. |

Accepted means the runtime acknowledged input. It does not mean the requested
work was completed. Submitted answers await native resolution. Rejected,
unavailable and unknown delivery remain distinct; unknown delivery and stale
turns are never retried automatically.

Ordinary Codex access remains available. Some spawned agents do not accept
direct input. Zoetrope explains that restriction and never silently forwards a
child's feedback to its coordinator. Select the coordinator explicitly when
that is the intended recipient. Claude sessions retain viewing and replay;
live control currently targets the compatible local Codex runtime on Unix.

Native questions require a current connection to confirm that the original
request is still pending. A saved request ID alone is insufficient. Questions
resolved elsewhere are retained with their drafts; expired requests can be
deliberately converted to feedback with F5. That conversion creates a draft and does
not claim the original question was answered. No answer or default choice is
submitted automatically. Native command and permission approvals stay in Codex.
Dismissal is local: it does not answer or cancel the runtime's request.

Secret answers remain hidden while editing and reading saved questions. They
cannot be copied into ordinary feedback; use `f` to write new non-secret
feedback if needed. Local state files are private to your user account.

## Documents and replay

Recorded Markdown changes appear with their producing agent. Open a selected
document to request its VS Code MD Reader preview. A missing file produces a
clear error; an unavailable bridge falls back to opening the exact file in VS
Code. The status says a preview was requested because OS dispatch has no render
acknowledgment. Bridge installation is described in
[the editor guide](../editors/vscode/README.md).

Historical question and document import recognizes supported structured records
within the bounded transcript index. Unsupported or unseen records are not
invented. Opening an old session retains the original graph and playback tools;
replay controls do not send live feedback.

```bash
zoe                         # global goal console
zoe --project               # follow the current project's session
zoe /path/to/project        # follow another project's session
zoe /path/to/recording.jsonl # replay a recording
zoe SESSION_ID              # replay a stored session, or a unique ID prefix
zoe SESSION_ID --follow     # open a stored session at its live edge
zoe inspect SESSION_ID      # print the original session report
```

## Local goal and work updates for agents

The terminal and agents share a local record. `zoe state --help` is the field
reference; [the reporting convention](AGENT-REPORTING.md) gives creation and
partial-update examples. Use the exact provider, full session ID and state
location supplied by the tracking request. A project path does not identify a
session. Replace `STATE_DIR`, `SESSION_ID` and `WORK_ID` below before running.
These commands update local Zoetrope records; they send no native input.

```bash
zoe state --root 'STATE_DIR' apply codex 'SESSION_ID' '{"op":"set_goal","title":"Ship the change","criteria":[{"id":"behavior","label":"Requested behavior works"},{"id":"checks","label":"Relevant checks pass"}],"reason":"Initial agreed scope","provenance":"operator"}'

zoe state --root 'STATE_DIR' apply codex 'SESSION_ID' '{"op":"verify","id":"checks","revision":1,"evidence":[{"kind":"reference","detail":"Relevant checks passed for the inspected commit","provenance":"agent"}]}'

zoe state --root 'STATE_DIR' context codex 'SESSION_ID'

zoe state --root 'STATE_DIR' apply codex 'SESSION_ID' '{"op":"upsert_work_item","expected_revision":1,"item":{"id":"WORK_ID","status":"done","blocker":null}}'
```

The partial work update assumes an existing item and the returned revision 1;
use its actual stable ID and current acknowledgment revision. Omitted fields
retain prior values; explicit null clears optional context. A reported `done`
status never verifies a criterion. Record actual evidence through `verify`.

`apply` returns a short acknowledgment. `context` excludes saved feedback,
questions and delivery history, emits at most 64 KiB, and shows omission/
truncation counts. Its default work page is 16; follow `next_work_offset` with
`--offset N --limit N` when needed (maximum limit 32). `show` deliberately reads
the full local record. These local reads incur no model generation; the working
agent's new reports still consume input/output/tool tokens.

State lives in `ZOETROPE_STATE_DIR`, or `XDG_STATE_HOME/zoetrope`, or
`~/.local/state/zoetrope`. `zoe state --root DIR ...` selects another state
directory. Writes use an OS lock and atomic replacement; Unix files are private
to their owner. Interrupted sends restore as unknown and are not replayed.

## Resource use and compatibility

The app, inventory, persistence and control are Rust. The VS Code bridge is
small and dependency-free. Passive monitoring uses no model calls, internet
service or telemetry. History metadata uses bounded incremental reads; full
local records are cached within a byte and entry limit. Full transcript replay
remains an explicit operation.

The original Mac installation and resource observations are dated 25 October
in [the earlier validation record](GOAL-CONSOLE-VALIDATION.md). They do not
establish installation or measurements for the new semantic-context code.
Current ticket-level Linux checks are linked from [the research index](research/README.md);
final combined verification is recorded separately from those historical results.

The earlier Mac probes on Codex 0.164 checked shared observation,
exact-turn steering and interruption, queue operations, and question
replay/resolution using disposable sessions. Second-client answer arbitration
and real command/permission approval coexistence have not been established by
those probes. Live actions fail visibly when the local runtime does not support
them; Zoetrope does not start a replacement daemon or change runtime settings.

Final semantic-context checks and resource observations are in [the 27 October validation](SEMANTIC-CONTEXT-VALIDATION.md). Native tracking and criteria drafts use the running executable and an absolute state root; both read compact context.
