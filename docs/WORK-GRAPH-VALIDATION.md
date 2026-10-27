# Work graph validation — 28 October 2026

The approved six-ticket iteration is implemented on `feat/goal-console` and
installed in the user-selected connected Linux environment. This record was
completed on 28 October in Europe/Amsterdam (27 October UTC). It supersedes the
earlier semantic iteration's measurements for the current application.

## Source and verification boundary

Final application/skill source: `2a65e3155310768a34833a4f0b762c4e6afc72df`.
Final installer polish: `f5e1d29ccd31d335ab4cc85f052a07fa8cceb813`, integrated at
`2b06e537c4c5256130380dea398893b9ef71beb1`. The polish changes the shell installer
and its public regression, not Rust application code or packaged skill content.
Subsequent completion changes are documentation and synthetic QA artifacts.

Checks ran locally on Linux x86_64, using Rust 1.99.0 and the declared Rust 1.88.0
minimum. GitHub reports no CI checks on this branch; local results below are not
a claim of hosted CI success. The separately locked browser workspace remains
excluded from the host workspace.

| Check | Result |
| --- | --- |
| Final full native all-feature suite, including installer polish | **373 passed**, one existing manual resource probe ignored in the ordinary run |
| Portable suite without default features | **189 passed**: 176 library and 13 public Console tests |
| Strict all-target/all-feature Clippy, Rust 1.99 and 1.88 | Passed, warnings denied |
| Portable strict Clippy | Passed, warnings denied |
| Minimum-Rust tracking/UI/runtime regressions | 95 passed; manual probe ignored |
| Normal parallel UI suite, both compilers | 69 passed on each; manual probe ignored |
| Formatting and patch hygiene | Passed |
| Separately locked wasm32 frontend | Passed |
| Rust documentation, warnings denied | Passed |
| Standalone local crate archive | Packaged and compiled; nothing uploaded |
| Final release build | 8,442,096-byte executable |
| Skill validation, packaged and installed | Passed |
| Public installer process tests | All eight passed; actual updater completed without warnings |
| Independent Standards / Spec review | Both clear after fixes |

Final commands included these public checks, with separate target directories
where compiler/feature isolation was useful:

```sh
cargo test --offline --locked --all-features
cargo test --offline --locked --no-default-features
cargo clippy --offline --locked --all-targets --all-features -- -D warnings
cargo +1.88.0 clippy --offline --locked --all-targets --all-features -- -D warnings
cargo clippy --offline --locked --no-default-features --all-targets -- -D warnings
cargo fmt --all --check
cargo check --offline --locked --manifest-path web/wasm/Cargo.toml --target wasm32-unknown-unknown
RUSTDOCFLAGS='-D warnings' cargo doc --offline --locked --all-features --no-deps
cargo package --offline --locked
cargo build --offline --locked --release --bin zoe
sh -n scripts/install.sh
git diff --check
```

Cargo's package allow-list excludes tests/benchmarks and emits the corresponding
notices; standalone package verification still compiled successfully. The new
native Unix `libc` dependency is optional and was already transitive. Portable
code gains no filesystem/runtime dependency. An initial wasm check through a
compiler alias without that standard library failed; the already installed
stable wasm target passed with the existing browser lockfile.

Raw local logs are retained under `.local/work-graph-validation/final/` and are
not required in a clone. Per-ticket RED/GREEN evidence and integration commits
are linked from the [research index](research/README.md).

## Behavioral and visual evidence

Public Console input/render, executable/files/process, owned PTY and independent
local protocol-peer checks exercise graph navigation, declared prerequisites,
cycles/missing references, stable selection/cameras, exact owner joins and
recipient-pinned drafts. They also cover source precedence, valid empty/removal,
stale last-good recovery, duplicate IDs, bounds, source opt-in/disable and exact
goal isolation. No private parser/layout tests were introduced.

Independent [Standards and Spec reviews](research/work-graph-review.md) preserved
their original findings and rechecked fixes separately. FIFO sources now fail
promptly without blocking the tracking lock; conflicting selected ticket IDs
retain a coherent last-good namespace. Typed status cues remove dependence on
presentation wording. Three pre-existing contention fixtures now identify the
exact public goal record rather than choosing the first lock in a directory
that also contains `tracking.lock`; their assertions remain intact. See
[review-fix evidence](research/work-graph-review-fixes.md) for the reproduction
commands, earlier parallel failure and final parallel passes.

The [skill exercise](research/wg-4-implementation.md#independent-disposable-behavioral-exercise)
used disposable projects to establish Matt-style setup, new-goal/resume, disable,
old-pointer/local-disable precedence and a seeded no-Matt fallback. The model
maintained ordinary artifacts; no per-task state JSON or automatic outcome credit
was needed. It sent no real native input and did not edit the active bot project.

The final installed executable was captured in an isolated tmux server at
150×42, 106×48 and 60×28, in readable/overview modes. Its environment included
`NO_COLOR=1`; `--color always` produced actual ANSI colors. The
[synthetic captures](research/wg-5-terminal-captures/README.md) and
[reproduction script](research/probes/wg-5-terminal-capture.py) show status colors,
non-color glyphs, selected work, prerequisites, compact context and separate
reported/verified progress. Earlier visual QA inspected all six views; final
installed-release inspection rechecked narrow readable and wide/tall overview
views. Pillow/tmux are QA tools, not application dependencies.

## Resource observations

The [public release probe](research/probes/work-graph-resource.py) used explicit
structured local sources, synthetic native metadata and an owned PTY. Each sample
followed three seconds of warmup and measured ten seconds. No live native control,
model or remote tracker polling was involved.

| Source graph | Declared edges | Observed open | RSS / peak RSS | CPU, one core | Idle terminal output |
| --- | --- | --- | --- | --- | --- |
| 12 tasks | 8 | 70 ms | 8,464 KiB (8.3 MiB) | 0 measured ticks | 0 bytes |
| 512 tasks, sparse | 508 | 74 ms | 18,712 KiB (18.3 MiB) | 0.1% | 0 bytes |
| 512 tasks across 16 dense/cyclic tables | 15,872 | 69 ms | 33,580 KiB (32.8 MiB) | 0.4% | 0 bytes |

The first two samples ran concurrently; the dense sample ran separately. CPU
sampling is coarse; zero ticks does not prove zero cost. Open time means the
Work header became visible, not whole-history parsing. These are observations,
not latency/memory guarantees for arbitrary busy projects or platforms.

The current graph bounds are 512 task nodes, 2,048 drawn dependency edges and
64 unresolved placeholders, with visible omissions. Tracking allows at most
16 sources per binding and 256 KiB per file, with bounded rows/cells/dependencies,
registries and caches. Large source relationships can exceed the drawn-edge
bound without pretending all edges were displayed. The committed
[raw observations](research/work-graph-resource-results.json) record source sizes,
release identity and qualifications.

The separate public debug history probe passed with 1,000 and 4,000 goals:
startup 313/837 ms and peak UI poll 19.505/81.677 ms, respectively. It checks
unchanged-view stability, not busy replay or universal interactive performance.
The ordinary-suite ignored probe was run explicitly and passed.

## Installed app, update and live observation

`~/.local/bin/zoe` and `zoe-update` are on PATH. The installed executable matches
the final release exactly: SHA-256
`2084c912cbd69147afa15e9d197a16059d479b136dfb29d4908b6cd0b5fb8408`.
The installed skill at `~/.codex/skills/zoetrope-tracking` matches the packaged
files and passes the skill validator. The installer and actual `zoe-update`
were run, including an update from `/tmp`. After portable-copy polish, the actual
updater completed without stderr warnings. It rebuilds this permanent checkout,
without fetching or switching branches; ordinary launches execute the binary.

The old attached `zoe-live` tmux session remains intact. A separate persistent
`zoe-work-graph` session runs the installed app with `--color always` and observes
the active `gymnasium-tg-bot` goal. At observation it showed 16 tasks, 17 declared
dependency edges, 13 reported done/3 active, and 1 of 5 verified outcomes with a
weighted 27% progress indicator. These are live reports and may change.

Only viewer navigation was sent. No guidance, queue, stop, tracker setup or
project-instruction edits were sent to that project. Its current work uses the
existing report fallback; the new persistent source skill has not been activated
there. An operator can deliberately review/send the new `T` setup request later.
Raw live session data remains local; committed screenshots use synthetic data.

Outside tmux, use `tmux attach -t zoe-work-graph`. From an existing tmux client,
press Ctrl-b then `:`, enter `switch-client -t zoe-work-graph` and press Enter.
Use `w` for Work, `o` for overview, `r` for readable selection, and Esc for Back.
The old process inherited `NO_COLOR=1`; explicit color restores output without
changing global environment settings.

## Completion and limits

All six tickets are implemented, reviewed and installed. The spec and tickets
are native GitHub issues #15–21, with blocking/sub-issue relationships. They close
through [PR #8](https://github.com/wagar-meilech/zoetrope/pull/8) when merged; main
has not been merged. The [frontier](WORK-GRAPH-PLAN.md#live-task-frontier) records
completed working status separately from remote issue closure.

Passive monitoring remains Rust/local and calls no model. Structured local task
sources refresh on the existing five-second background worker; arbitrary prose
and remote ticket changes are not automatically interpreted. The opt-in skill
connects existing conventions once, and agents keep their normal artifacts current.
Source completion never verifies outcomes. Historical native/Codex compatibility
limits still apply; owned peers and this live observation do not establish every
runtime, terminal, operating system or model's instruction compliance.
