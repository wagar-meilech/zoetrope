# Zoetrope customization questionnaire

All five decisions below have been **answered**. The checked choices and custom answers are the source of truth. See [the implementation review](IMPLEMENTATION-REVIEW.md) for the proposed spec and remaining test-scope confirmation.

Mark your choices with `[x]`, write your own answers, or edit the wording directly. You can answer some now and return to the rest later. Save this file when you finish editing.

## Already established

- Keep the interface in the terminal, using Zoetrope's interactive graph.
- Provide an overview across projects and sessions, including their subagents.
- Make steering work a central interaction.
- Show progress toward the user's goal and use plain-language activity labels.
- Keep additional model-token usage low.
- **Persistent questions:** unanswered questions must remain available until you choose to answer or deliberately dismiss them. Preserve partially completed answers when navigating between sessions or reopening Zoetrope; do not silently submit defaults.

## Q1 — What appears on launch?

Should `zoe` open a global graph of active and waiting sessions across every project, expandable into subagents, with older/completed sessions behind a toggle?

**Recommendation:** active work first, with a history toggle. Keep ongoing work visible without burying it in historical sessions.

- [ ] Yes — active work first, with a history toggle. **Recommended.**
- [ ] Show every session, including completed ones, by default.
- [X] Use my custom answer below.


**My answer, changes, or notes:**

<!-- Yes, but that toggle should be essentially a single button press and the global interface of everything that has happened before as a single (ideally searchable graph)-->

Yes, but that toggle should be essentially a single button press and the global interface of everything that has happened before as a single (ideally searchable graph)

## Q2 — What earns progress?

Should each goal have editable completion criteria, with progress advancing when the agent records verification evidence, including tests and final checks? The display would show a progress bar and “3 of 5 outcomes verified,” with scope changes visible.

**Recommendation:** use completion criteria and recorded evidence. New work can revise the plan visibly. The percentage represents checklist completion, not a prediction of remaining time.

- [X] Yes — use completion criteria and recorded evidence. **Recommended.**
- [ ] Use the agent's estimated percentage instead.
- [ ] Use my custom answer below.

**My answer, changes, or notes:**

<!-- Write here. -->

## Q3 — How do you steer?

Should selecting a goal or agent open a persistent feedback box for that target, with separate **Send guidance**, **Queue for later**, and **Stop** actions, plus a visible recipient and delivery status?

**Recommendation:** steer the selected target inside Zoetrope. Guidance goes only to that target unless you select the whole goal. Clearly distinguish queued feedback from delivered feedback.

- [ ] Yes — steer the selected target inside Zoetrope. **Recommended.**
- [ ] Keep feedback in the original Codex terminal.
- [X] Use my custom answer below.

**My answer, changes, or notes:**

<!-- Write here. -->
Yes, but I think it should still be accessible through Codex, plus, ideally not burn too many extra tokens

## Q4 — How much explanation?

Should each meaningful batch of work have a short, plain-language purpose, with its Bash commands expandable underneath? For example: **“Checking recovery after a dropped connection”**, followed by the commands used to check it.

**Recommendation:** explain each batch and reuse existing commentary where possible. Generate a brief purpose when activity changes; avoid a second model summarizing every command.

- [ ] Yes — explain each batch, with commands underneath. **Recommended.**
- [ ] Give every individual Bash command its own explanation.
- [X] Use my custom answer below.

**My answer, changes, or notes:**

<!-- Write here. -->
Yes, besides the most technical stuff - like we should give semantic purpose to things that are valuable to keep in mind, however giving a semantic оболочка to stuff like file reads/writing to file is a bit too excessive

## Q5 — When should work wait for you?

Should agents keep working while you watch and interject, with an optional setting to pause at milestones and prominent highlighting when your input is needed?

**Recommendation:** continuous work by default, with optional milestone checkpoints.

- [ ] Yes — continuous work, with optional checkpoints. **Recommended.**
- [ ] Pause at every milestone until I respond.
- [X] Use my custom answer below.

**My answer, changes, or notes:**
Yes, as few interruptions as possible, with the option to steer as it's happening

<!-- Write here. -->

## Anything else

Add examples, desired shortcuts, interaction preferences, or changes to the established requirements here.

<!-- Write here. -->
Mentioned one thing in the chat already, but also it would be nice for it to be able to point to the docs that agents create and for that to be accessible as well, like maybe it can open vscode on those files with the md-render plugin I have on it as well
