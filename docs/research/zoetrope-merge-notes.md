# Goal-console integration merges

## ZOE-5 document preview

Merged branch `feat/zoe-5-preview` (tip `6e0d0c8`) into `feat/goal-console` as `e6962d8` using a non-squash merge. No conflicts.

Validation on integration using its root target directory:

- `cargo fmt --all --check`: passed.
- `cargo test --locked --all-features`: 207 library, 9 binary and 5 document-launch tests passed.
- `cargo check --lib --locked --no-default-features`: passed.

## ZOE-1 fleet foundation

Merged branch `feat/zoe-1-fleet` (tip `b728161`) as `dfa63cf` using a non-squash merge. `src/lib.rs` auto-merged preserving portable `console` and native-only `documents` exports. No conflicts; no source changes beyond merges.

Validation on combined integration:

- `cargo fmt --all --check`: passed.
- `cargo test --locked --all-features`: 231 tests passed (207 library, 9 binary, 10 console, 5 document-launch).
- `cargo check --lib --locked --no-default-features`: passed.

Working tree is clean. No push, PR creation, release build, installation or model/runtime mutation performed. Known pre-existing clippy warnings remain assigned to root for final integration cleanup.

## ZOE-3 local Codex control

Merged final branch `feat/zoe-3-control` (tip `4bf1ab1`) into integration `c84a0bf` as `fa10ab0`, after implementer confirmed completed lint/docs/test checks. Non-squash merge, no conflicts. Native control export, early CLI dispatch and optional native-only tungstenite dependency preserved. No unrelated source changes.

Validation on integration:

- `cargo fmt --all --check`: passed.
- `cargo check --lib --locked --no-default-features`: passed.
- `cargo test --locked --all-features`: all 244 passed (207 library, 9 binary, 13 external Codex protocol-peer, 10 console, 5 launcher). This command used scoped sandbox escalation because macOS local Unix socket binding is blocked inside the sandbox. Test peers use disposable sockets and never connect to the installed runtime.

Implementer additionally passed warnings-as-errors clippy and rustdoc after merging integration's style cleanup, as recorded in `docs/research/zoe-3-implementation.md`. No source conflict requiring rerun of those checks; final combined integration will repeat them after remaining features. Working tree clean. No push, install or live model/runtime mutation performed.

## ZOE-2 durable work records

Merged final `feat/zoe-2-state` tip `cefe2bb` into integration `fa10ab0` as `52a1766`. Non-squash merge, no conflicts. The ticket had already integrated the latest control work; both `workstate::run_cli` and `control::run_cli` dispatches are preserved. Native workstate/control/document exports and portable console export are present. Optional native fs2 locking dependency preserved.

Validation on combined integration:

- `cargo fmt --all --check`: passed.
- `cargo check --lib --locked --no-default-features`: passed.
- `cargo test --locked --all-features`: all 258 tests passed (207 library, 9 binary, 13 control peer, 10 console, 5 launcher, 14 workstate). Test run used the scoped Unix-socket peer escalation; no installed runtime was touched.

Implementer also passed warnings-as-errors clippy and rustdoc on its final tree containing integration `fa10ab0`. No merge conflict/source edits requiring extra targeted checks. Root tree clean; no push, install or model/runtime mutation. This unblocks terminal integration ZOE-4.

## ZOE-4 terminal graph, steering and runtime

After root's explicit final validation confirmation, merged combined `feat/zoe-4-runtime` tip `89a1c6c` into clean integration `3c38d7a` as **`d853af2`** with Conventional subject `feat(ui): integrate terminal goal console`. Non-squash merge, no conflicts or source tweaks. Combined branch includes UI `e600ee7` and latest root integration.

Validation on merged root:

- `cargo fmt --all --check`: passed.
- `cargo check --lib --locked --no-default-features`: passed.
- `cargo test --locked --all-features`: **278 passed**, no failures (207 library, 8 binary, 13 Codex control, 10 console, 5 document launcher, 5 executable/PTY, 16 fleet UI, 14 workstate). Actual PTY cases passed together in5.37s; public UI16 in0.59s. Scoped escalation allowed isolated local Unix-socket protocol peers/PTYs; no installed runtime was mutated.

Implementer had already passed strict warnings-as-errors clippy, rustdoc, formatting and portable library checks at the exact final source tip. Final combined code review/install/PR work belongs to root. Optional `cargo test --no-default-features --lib` reported an unchanged upstream test-only handler cfg issue at `src/state/mod.rs:1230`; documented for review, no unrelated fix performed during merge. Required portable library build passes.

Root tree clean. No push, install, main merge or Git-history normalization performed. Full notes at `zoe-4-runtime-implementation.md` and `zoe-4-implementation.md`.

## Final review fixes

After final clean implementer confirmation, merged `feat/zoe-review-fixes` tip `8e58b8b` (source `a1ed060`, latest integration docs `b454397` included) into root as **`47b41e1`** (`fix(ui): integrate final goal console review fixes`). No conflicts or source tweaks.

Independent merged checks: fmt passes, portable core passes, all290 native tests pass (one manual resource benchmark ignored), all176 portable library tests pass. Native tests use scoped local Unix-peer escalation; no live installed-runtime actions.

Independently re-reviewed R1–R8 plus follow-up delayed-copy/orphan-edge/held-F2 cases. All cleared with source evidence and public regression/probe results. Resolution appended to `docs/research/zoetrope-final-review-2026-10-25.md`. Root handles final release/installed-QA/docs/wasm/packaging/PR/history lint; merger performed no push, install, history normalization or main merge. Tree clean at47b41e1.
