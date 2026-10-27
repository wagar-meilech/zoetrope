Opening `zoe` now shows goals and agents across local projects in one searchable terminal graph. Progress comes from editable completion criteria and recorded evidence. Persistent, exact-recipient feedback and questions add steering while existing session replay and ordinary Codex access remain available.

The app stays Rust, with a small dependency-free VS Code bridge for explicit Markdown previews. Passive monitoring makes no model calls. Inventory and state use bounded caches and background IO; graph placement uses batched geometry.

Validation on macOS arm64:
- 290 native tests and 176 portable library tests passed, including real executable/PTY and independent protocol-peer cases.
- Strict clippy, formatting, portable compilation, WebAssembly check, rustdoc, spelling, commit lint and standalone Cargo packaging dry run passed.
- The installed release was exercised against real local history, including replay/return and an external progress update without navigation. Real MD Reader preview was verified.
- Fresh passive monitoring observed about 0.2% average process CPU and 8.2–8.4 MiB RSS; after full replay, about 23.4 MiB RSS. These are short local observations. A 4,000-goal debug fixture measured 108.5 ms peak UI polling.

Installed Codex 0.164 probes verified shared observation, exact-turn guidance/interruption, queue operations and question replay/resolution. Some child/helper threads reject direct input; the console explains that and never silently redirects to a parent. Real second-client answer arbitration and command/permission approval coexistence were not established by those probes. Native approvals stay in Codex.

[Usage guide](https://github.com/wagar-meilech/zoetrope/blob/feat/goal-console/docs/GOAL-CONSOLE.md) · [Validation and compatibility](https://github.com/wagar-meilech/zoetrope/blob/feat/goal-console/docs/GOAL-CONSOLE-VALIDATION.md) · [Approved spec](https://github.com/wagar-meilech/zoetrope/blob/feat/goal-console/docs/GOAL-CONSOLE-SPEC.md)

Closes #1
Closes #2
Closes #3
Closes #4
Closes #5
Closes #6
Closes #7
