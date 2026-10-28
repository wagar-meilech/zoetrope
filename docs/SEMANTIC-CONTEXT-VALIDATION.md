# Semantic context validation — 27 October 2026

The semantic continuation is implemented and verified locally on `feat/goal-console`. The final checked implementation commit is `c318c1c8`; integration `856c565` contains identical application source. Subsequent completion artifacts change documentation/probes only.

This record concerns Linux x86_64 with Rust 1.99.0, plus strict compilation/linting on the declared Rust 1.88 minimum. The earlier [25 October Mac installation](GOAL-CONSOLE-VALIDATION.md) remains historical. This source-validation run preceded installation and publication. See the [installation record](research/local-installation.md) and [task graph](SEMANTIC-CONTEXT-TICKETS.md) for subsequent actions. No merge into main is authorized.

## Checks

| Check | Result |
| --- | --- |
| Full native all-feature tests | **319 passed**, 1 manual resource probe ignored in the ordinary run |
| Portable library tests without default features | **176 passed** |
| Strict all-target/all-feature Clippy, Rust 1.99 | Passed with warnings denied |
| Strict all-target/all-feature Clippy, Rust 1.88 | Passed with warnings denied |
| Formatting / patch hygiene | Passed |
| Separately locked browser frontend, wasm32 | Passed |
| Public Rust documentation, warnings denied | Passed |
| Optimized native release | Built; 7,585,584-byte Linux executable |
| Standalone local crate archive | Packaged and compiled successfully; nothing published |
| Manual 1,000 / 4,000-goal resource probe | Passed, including stable unchanged rendering |
| Independent Standards / Spec review | Both clear after fixes |

The exact implementation tests and strict commands are in [review-fix evidence](research/semantic-context-review-fixes.md). The merger reran the affected public regressions and verified source-tree identity; [integration evidence](research/semantic-context-integration.md) records each merge. Portable/browser checks cover the final shared code; the last bootstrap changes are native-only.

Final commands included:

```bash
cargo test --locked --all-features --quiet
cargo test --locked --no-default-features --lib --quiet
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo +1.88.0 clippy --locked --all-targets --all-features -- -D warnings
cargo fmt --all --check
cargo check --manifest-path web/wasm/Cargo.toml --locked --target wasm32-unknown-unknown
RUSTDOCFLAGS='-D warnings' cargo doc --locked --all-features --no-deps
cargo build --release --locked
cargo package --locked
```

Packaging retains the existing source allow-list, so Cargo notes that tests and benchmarks are excluded from the archive. Verification compiled the standalone package. No dependencies or manifests changed.

## User-visible verification

Public console, executable, local protocol peer and owned-terminal checks cover work-first inspection, source/time, explicit dependencies and stale outcome links, actual ownership, pinned feedback during reassignment, separate guidance/queue/stop capability, persistent drafts, narrow rendering, and two mouse Replay/return cycles without live mutations.

Review regressions establish that an empty native plan withdraws old work, relative state roots resolve to the same absolute store from another project, and an incompatible global `zoe` cannot replace the running executable in native tracking/criteria drafts. Both requests use filtered compact context; criteria requests exclude saved secret answers, private drafts and delivery history. Full `show` remains an explicit operator action.

Manual public-renderer inspection at 150×42 and 60×28 confirmed visible work, owner, purpose, next action and steering availability. [Synthetic screen captures](research/probes/semantic-context-screens.txt) record these views. Their goal/evidence data is explicitly synthetic.

## Resource observations

The optimized public terminal-runtime helper used an isolated fixture with one goal, three explicit work items and two native plan producers. Its Unix control socket was deliberately unavailable; it read no real native sessions and made no model calls.

| 16-second idle observation | RSS / peak RSS | Reported process CPU | Idle terminal output |
| --- | --- | --- | --- |
| State/steering foundation | 7,568 KiB | 0 clock ticks | 0 bytes |
| Final semantic runtime | 7,728 KiB (about 7.5 MiB) | 0 clock ticks | 0 bytes |

Each sample starts after three seconds of initialization and spans 16.001 seconds. No measured clock ticks means CPU use was below the sampling resolution; it does not prove universal zero CPU. These small idle fixtures do not measure busy model runs, a loaded live runtime or full transcript replay.

| Archived goals | Baseline probe elapsed | Final probe elapsed | Baseline peak UI poll | Final peak UI poll |
| --- | --- | --- | --- | --- |
| 1,000 | 307 ms | 315 ms | 18.954 ms | 19.690 ms |
| 4,000 | 767 ms | 795 ms | 78.329 ms | 80.625 ms |

The history probe reports elapsed startup/discovery/render work together with its 100 ms unchanged-view observation. Results are individual debug-build fixture samples, not thresholds or universal performance guarantees. Both runs preserved stable rendering after unchanged refresh.

[Raw observations](research/semantic-context-resource-results.json) and [probe sources](research/probes/semantic-context-resource.rs) make the boundaries explicit. The [fixture builder](research/probes/semantic-context-fixture.py) uses the actual executable and a new empty directory. The [measurement driver](research/probes/semantic-context-resource.py) samples the owned process through Linux `/proc`.

To repeat, build the release, compile the small Rust runtime helper against its `libzoetrope.rlib` and existing `tokio`/`anyhow` release dependencies, create the fixture in a fresh directory, then pass the helper and fixture paths to the Python driver. This requires no extra Python packages or model access.

## Review and remaining external action

[The two-axis review](research/semantic-context-review.md) preserves initial findings and final resolutions separately. Standards has zero remaining findings; Spec has zero remaining findings. The source still uses recorded/public data and explicit reports, with no helper model, automatic project-instruction changes or scheduling changes.

All four semantic tickets are implemented and verified. Automatic review initially blocked publication; the user then explicitly approved the payload and destination on 27 October 2026. The [spec](https://github.com/wagar-meilech/zoetrope/issues/9) and tickets [#10](https://github.com/wagar-meilech/zoetrope/issues/10), [#11](https://github.com/wagar-meilech/zoetrope/issues/11), [#12](https://github.com/wagar-meilech/zoetrope/issues/12), [#13](https://github.com/wagar-meilech/zoetrope/issues/13) are published and close through PR #8 when merged.
