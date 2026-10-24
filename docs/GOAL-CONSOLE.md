# Using the goal console

> Implementation is in progress. The foundations below are integrated; the terminal
> interface is unfinished and the installed `zoe` has not yet been rebuilt.
> See [the saved checkpoint](PAUSED-CHECKPOINT.md) for exact status.

Run `zoe` to see work across local projects. The native app groups projects,
goals and agents into an expanding graph. `H` switches between Current and All
history; `/` searches the same graph; `?` shows keyboard help. Selection stays
with the same work as the inventory refreshes.

Current emphasizes working agents and requests needing attention. Without a
compatible live connection, recent transcript writes are a fallback signal;
they do not prove that an agent is running or that its goal is finished. All
history includes locally available archived Codex sessions. Search covers
indexed names, goals, criteria and recorded document paths, rather than every
tool output in every transcript.

## Goal progress

A goal's completion criteria define its progress. Each verified criterion needs
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

Milestone checkpoints are off by default. Enabling the local preference affects
the criteria-request draft; sending that guidance asks the agent to check in.
The preference alone does not pause Codex or change its permission rules.

## Steering and questions

The composer belongs to the selected recipient. Drafts and partial answers stay
available across navigation and restart. Check the recipient before sending.

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
deliberately converted to feedback. That conversion creates a draft and does
not claim the original question was answered. No answer or default choice is
submitted automatically. Native command and permission approvals stay in Codex.

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

## Local goal updates for agents

The terminal and agents share a local record. `zoe state --help` lists the
supported updates. Use the exact session ID; these commands never edit native
transcripts or send input to Codex.

```bash
zoe state apply codex SESSION_ID '{"op":"set_goal","title":"Ship the change","criteria":[{"id":"behavior","label":"Requested behavior works"},{"id":"checks","label":"Relevant checks pass"}],"reason":"Initial agreed scope","provenance":"operator"}'

zoe state apply codex SESSION_ID '{"op":"verify","id":"checks","revision":1,"evidence":[{"kind":"reference","detail":"Relevant checks passed for the inspected commit","provenance":"agent"}]}'

zoe state show codex SESSION_ID
```

Use actual evidence when verifying a criterion. `apply` returns a compact saved
acknowledgment so routine updates do not echo the entire history into an
agent's context. `show` explicitly reads the complete local record.

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

The installed Codex 0.164 runtime has been checked for shared observation,
exact-turn steering and interruption, queue operations, and question
replay/resolution using disposable sessions. Second-client answer arbitration
and real command/permission approval coexistence have not been established by
those probes. Live actions fail visibly when the local runtime does not support
them; Zoetrope does not start a replacement daemon or change runtime settings.
