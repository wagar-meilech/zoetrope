# ZOE4 runtime / executable implementation

Worktree /Users/melikhov/Repos/zoetrope-worktrees/zoe-4-runtime; branch feat/zoe-4-runtime. Confirmed clean integrationc07fb01 before exploration. Read CLAUDE, approved spec/task graph, TDD/tests/mocking, codebase-design and root/UI handoff notes. No glossary/ADR files found. Merged stable library checkpoint2b4b5ec with a Conventional merge message before first product/test slice.

Ownership: main.rs, new src/fleet/runtime.rs, new executable/PTY seam tests/support helper. UI agent owns parent fleet module and other workers; runtime owns only module export line there. Child module can read parent-private cached fields. Agreed replay_source() returns selected cached SessionKey/path, never performs IO. One EventStream owned directly by native runtime, never nesting old tui::run.

Use CARGO_TARGET_DIR=/Users/melikhov/Repos/zoetrope/target/zoe-4-runtime, Rust1.88. No models, daemon, settings, native session writes, installation or pushes.

## Vertical TDD

1. Actual executable bare launch opens global fleet and quits cleanly. Added tests/fleet_runtime.rs + stdlib external PTY driver. RED: existing binary showed no Current work header (fleet=false,exit0);5.08s, exit101. Minimal GREEN implementation now running: Cli::Fleet default; event-driven run with one directly owned EventStream and cached Notify rendering. Replaced obsolete private bare-cwd unit test with this executable seam test. Other legacy parser/replay tests remain.

## Lifecycle design

Fleet redraws only input/cached changes; a timer will exist only while an edit is dirty or replay animation is active. Replay owns App+tailer channel/task and same input owner. Closing replay drops channels and keeps at most one retiring feeder; this prevents multiple expensive spawn_blocking full parses accumulating if repeatedly reopened. Replay request comes only from acknowledged cached source. All disk/socket loading remains on workers. Shutdown flushes native state workers off rendering thread after restoring terminal.

1. GREEN: actual bare fleet/clean quit passed0.21s.
2. --project preserves current-project session view. RED: unknown flag, exit1; GREEN after explicit legacy switch, actual fixture goal visible and no fleet header. Initial fixture had a macOS /var→/private/var cwd spelling mismatch, corrected fixture's stated cwd to the real OS cwd. Both executable cases passed together0.19s. Commit875d36c.
Merged UI layout checkpoint0194965; resolved only module-list conflict by preserving presentation and runtime exports. Native replay guard can now include show_help.

3. Read-only replay lifecycle. RED actual R did not enter replay. Implemented cached source -> existing App/handler/ui/tailer, with one directly owned EventStream, q/Q returning to the preserved FleetConsole, and at most one retiring replay loader. The real PTY exposed Ratatui 0.30 Terminal::clear synchronously querying cursor position (input ownership conflict); fullscreen resize invalidates buffers without that query. GREEN three executable cases together10.33s, external observe-only peer accepted no fleet mutations during two replay cycles. State CLI confirms the selected goal draft remains `Keep H/q R draft` and outbox stays empty. Commit191a6dd.

Merged final UI852ba25/e600ee7, including integration3c38d7a. Runtime child interfaces unchanged.

4. Autosave while user thinks. RED actual executable left autosaved=false without another keypress. Added a500ms deadline only while dirty_since is Some; a full state-worker queue retries at25ms rather than a hot expired deadline. GREEN feedback_autosaves_without_another_keypress1.16s. External state CLI observes persistence before any further terminal input/shutdown. Commit255da9b. Clean fleet has no per-frame timer.

5. Terminal paste lifecycle. RED multiline content saved through the UI but the terminal never received bracketed-paste enable/disable requests. Enable on terminal entry and restore on every return via guard. GREEN executable bracketed-paste case1.10s, literal H?qR/newline feedback survives, Ctrl-C exits cleanly, paste mode disabled. Commit89a1c6c.

The PTY adapter uses only Python stdlib and real isolated temporary files/socket peer. ANSI diff output can skip unchanged blanks, so feedback word checks compact whitespace; full rendered cell layout is exercised at the agreed public Console render seam. It uses actual state CLI JSON for durable draft assertions. No native Codex writes/models, daemon, or installation.

## Final verification / handoff

Clean branch feat/zoe-4-runtime at89a1c6c. Includes UI finale600ee7 and latest integration3c38d7a (`git merge-base --is-ancestor feat/goal-console HEAD` exits0). No install/release/push performed.

All commands ran on Rust1.88 with CARGO_TARGET_DIR=/Users/melikhov/Repos/zoetrope/target/zoe-4-runtime:

- `cargo test`:278 passed (207library,8binary,63integration),0failures. Actual PTY5 cases passed together5.41s; public UI16 passed0.60s.
- `cargo clippy --all-targets -- -D warnings`:pass,0warnings.
- `cargo fmt --all -- --check`:pass.
- `cargo check --no-default-features --lib`:pass, portable core preserved.
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps`:pass.
- `git diff --check` and clean status:pass.

An optional broader command `cargo test --no-default-features --lib` exposes an existing test-only gating problem: src/state/mod.rs:1230 calls crate::handler while native is disabled. Git blame confirms upstream39d6ea3, unchanged by this feature. Required portable library build passes; no state/replay contract was changed to hide that unrelated issue.

Replay cancellation keeps one retiring feeder until the existing full-file spawn_blocking parse completes, so returning to fleet is immediate but another replay waits for that previous loader. Final process shutdown waits for that one existing loader rather than detaching it or accumulating uncancellable parses. All file/socket observation is on workers. Fleet rendering has no idle16ms interval;16ms ticks are scoped to active replay playback/camera motion.
