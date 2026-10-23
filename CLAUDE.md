# zoetrope

Terminal UI that visualizes Claude Code and Codex agent sessions as a live flow graph. The same core also runs in the browser, compiled to wasm. Read-only, zero network.

## Approved goal-console work

The user approved `docs/GOAL-CONSOLE-SPEC.md`, its two public testing seams, and the task graph in `docs/GOAL-CONSOLE-TICKETS.md`. For this feature, explicit local Codex controls and opening a selected document extend the old read-only contract. Native transcripts stay read-only; the console must not add internet communication, telemetry, a hidden summarizing model, or automatic permission-policy changes. Preserve portable replay and the existing browser frontend.

Use GitHub Issues in `wagar-meilech/zoetrope`; details are in `docs/agents/issue-tracker.md`. The umbrella spec is issue #1. Integrate implementation worktrees into `feat/goal-console`, and use one PR to close the spec and tickets. Do not merge that PR into main without a user request.

Follow vertical TDD at the approved public console and executable/external-launch seams. The baseline is 207 library tests and 9 binary tests. Record red/green commands in the ticket's implementation notes. Do not test private implementation details or duplicate an existing native Codex run to simulate control.

The user explicitly requires fast, resource-light code. Keep the main implementation in Rust; JavaScript/TypeScript is limited to the small VS Code bridge. Use bounded metadata reads, incremental caches, lazy expansion and off-UI-thread local IO. Avoid heavy frameworks, background model summarizers, whole-history work on each frame and needless idle redraws. Verify representative resource behavior instead of assuming that a language guarantees efficiency.

Transcript formats enter through one boundary: `src/provider/` turns records into the facts in `src/fact.rs`, and nothing past it knows the format. See `docs/ARCHITECTURE.md` §0 before adding to either side.

See `docs/ARCHITECTURE.md` for the invariants and principles, `docs/DESIGN.md` for the module map and transcript format, `docs/HERDR-PLUGIN.md` for the Herdr bridge in `herdr-plugin/`, `TODO.md` for the roadmap, and `README.md` for usage.

## Running

Two crates: `zoetrope` at the root (the published lib + the `zoe` binary) and `zoetrope-web` in `web/wasm/` (the browser frontend, `publish = false`, wasm32-only). The browser frontend is **excluded** from the root workspace and resolves as its own — it cannot be compiled for the host at all, so keeping it a member would mean permanent phantom errors in the editor. The commands below therefore only touch `zoetrope`; the browser frontend needs its own manifest path.

```bash
cargo build                     # native (default features)
cargo clippy                    # lint — must pass with no warnings
cargo fmt                       # format
cargo test                      # test

# the browser frontend (trunk → web/public/wasm/), lint it explicitly:
bash web/scripts/build-wasm.sh
cd web/wasm && cargo clippy   # its .cargo/config.toml defaults to wasm32
```

## Commits

[Conventional Commits](https://www.conventionalcommits.org/), lowercase imperative subject. Scope is a module, not a file: `fact`, `provider`, `state`, `graph`, `timeline`, `tailer`, `ui`, `panel`, `cli`, `wasm`, `web`, `herdr`, `api`, `docs`. `herdr` is the Herdr plugin in `herdr-plugin/`, a bridge of shell scripts that opens the focused agent pane's session in `zoe`; it ships by git clone rather than with the crate. `api` is a change to what the crate exposes rather than to one module. A change inside one provider is `provider` (e.g. `fix(provider): inherit the timestamp across progress records`).

```
feat(timeline): index the playhead by event instead of wall-clock
fix(tailer): fold appends at the live edge without rebuilding
```

`cliff.toml` sets `filter_unconventional = true` — a non-conforming commit is dropped from the changelog. Lint with `committed HEAD~1..HEAD`, generate with `git cliff -o CHANGELOG.md`.
