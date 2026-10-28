# ZOE-1 implementation

Branch: `feat/zoe-1-fleet`, worktree `/Users/melikhov/Repos/zoetrope-worktrees/zoe-1`, initially based on `91081ad`.

Approved seam: public fleet console using real-format transcript files in isolated temporary provider homes, public commands, visible graph output and ratatui rendering. No private-method or internal-call-count assertions.

All commands below use `CARGO_TARGET_DIR=/Users/melikhov/Repos/zoetrope/target` and locked dependencies.

## Vertical TDD record

1. `cargo test --locked --test console opening_the_console_discovers_projects_from_both_providers`
   - RED: missing public `zoetrope::console` module.
   - Implemented explicit provider roots and console rendering. The fixture initially used an incorrect Claude title property (`title`); correcting to the real-format `aiTitle` made the existing provider extract it.
   - GREEN: one test passed. I briefly drafted the next test before this result completed, removed it until the first slice passed, then restored it and observed its RED separately.
2. `cargo test --locked --test console history_includes_archived_work_without_making_it_current`
   - RED: missing `ConsoleCommand` and `command`.
   - Added current/history navigation, flat Codex archived discovery and malformed-file tolerance.
   - GREEN: both console tests passed.
3. `cargo test --locked --test console expanding_a_goal_keeps_nested_agents_and_internal_reviews_under_their_owner`
   - RED: missing public graph and expansion/reveal commands.
   - Added stable graph identity/parent links and provider metadata; explicit ancestry now classifies Codex guardian reviews as children. Review helpers are hidden until revealed and never become independent goals.
   - GREEN: all three console tests passed.

## Public interface

`console::{Console, DiscoveryRoots, SessionKey, AgentKey, ConsoleCommand, FleetNodeId, FleetNode, FleetGraph}`. Serializable session/agent keys qualify provider and session, preserving the existing `main` root fact identity. Provider serializes as lowercase `claude` / `codex`.

Construction performs no IO; callers explicitly refresh local inventory. `Console::graph()` supplies visible nodes and parent edges. `Console::render()` supplies plain ratatui text for simple frontends. Terminal default-launch wiring remains ZOE-4.
4. `cargo test --locked --test console refresh_keeps_the_selected_agent_in_its_own_session`
   - RED: missing selection and selected-detail interface.
   - Added explicit graph selection; agent identity includes its provider-qualified owning session.
   - GREEN: four tests passed.
   - Tooling correction: sharing the top-level Cargo target directory across concurrent worktrees mixed root-library artifacts and produced a false missing-module error. All subsequent runs use `/Users/melikhov/Repos/zoetrope/target/zoe-1`; portable checks use the separate `zoe-1-core` directory. The parent approved this isolation.
5. `cargo test --locked --test console search_finds_a_collapsed_agents_name_and_preserves_the_selected_goal`
   - RED: missing search command.
   - Added case-insensitive indexed metadata search with contextual session/agent visibility and pinned selection even when hidden by a filter.
   - GREEN: five tests passed. A parallel-test temporary-home naming collision was found and fixed by adding a process-local atomic counter to the fixture names.
6. `cargo test --locked --test console large_history_keeps_its_latest_title_and_indexes_new_metadata`
   - RED: a title following a 1 MiB record was invisible.
   - Added cached provider classification, bounded head/tail metadata sampling, complete-record append offsets, truncation/rewrite handling, and stat-only unchanged-file refreshes. No full historical model is constructed.
   - GREEN: six tests passed.
7. `cargo test --locked --test console expanding_a_claude_goal_discovers_its_agents_without_opening_replay`
   - RED: Claude child/sidecar metadata was absent after expansion.
   - Added lazy Claude related-path discovery and sidecar label merging.
   - GREEN: seven tests passed.
8. `cargo test --locked --test console codex_history_uses_searchable_native_thread_names_and_observes_renames`
   - RED: native thread names were absent from history/search.
   - Added compact `session_index.jsonl` name indexing, then append-only updates after the initial index read.
   - GREEN: eight tests passed.
9. `cargo test --locked --test console graph_keeps_each_projects_goals_together_when_recent_writes_interleave`
   - RED: an older compiler goal appeared beneath the intervening editor project.
   - Added parent-based display traversal, preserving recency within each group.
   - GREEN: nine tests passed.
10. `cargo test --locked --test console an_indexed_claude_agent_stays_searchable_after_collapsing_and_refreshing`
    - RED: refresh evicted the lazily indexed child name after collapsing its goal.
    - Kept compact child metadata while its root exists; new child discovery remains lazy.
    - GREEN: ten tests passed.

## Review and efficiency

Reviewed with the engineering code-review skill. Replaced quadratic parent/sidecar lookups with indexed maps. `Console::graph()` now returns `&FleetGraph`, a cached borrowed view; rendering frames require neither filesystem discovery nor graph reconstruction. Selection commands do not rebuild the graph. Session/agent/node identities and compact graph views all derive serialization.

Transcript caches hold provider classification, a per-file parser and short metadata, not transcript bodies or replay models. Unchanged files are statted only. Initial transcript previews read at most 64 KiB of head plus 64 KiB of tail; Codex's identifying first metadata record is separately read in full once because it may embed large native instructions. An append consumes at most 64 KiB of new complete records, falling back to the tail for large bursts. Historical entries are never promoted merely by parsing them. Current mode uses writes in the preceding 120 seconds as a documented local recency fallback, not runtime-active or completion evidence; archived roots stay in history. Real runtime status belongs to ZOE-3/4.

A read-only Rust debug-build benchmark exercised only the public Console interface with 1,000 archived sparse rollout fixtures, each 16 MiB (16,777,216,000 logical history bytes). It measured initial inventory at 1,948.491 ms; twenty unchanged refreshes averaged 18.430 ms; 100,000 borrowed graph reads took 0.589 ms; history held 1,001 visible nodes. This is local debug/sparse-fixture evidence, not a production latency guarantee. Source: `docs/research/probes/zoe-1-inventory-bench.rs`; runner: `.local/research-artifacts/tmp/zoe-1-inventory-bench`. The fixture was deleted after measurement. Directory enumeration and file stats occur only on explicit refresh, never per graph access/frame.

Review also corrected archived explicit-path expansion to use the Codex storage root rather than treating its flat directory as a dated tree. Existing provider/replay contracts remain intact.

## Validation before integration merge

- `cargo test --locked --all-features`: 207 library + 9 binary + 9 console tests passed before the final cached-Claude regression. Final affected console run: all 10 passed.
- `cargo check --lib --locked --no-default-features`: passed.
- `cargo clippy --locked --all-targets --all-features -- -D warnings`: only two unchanged baseline findings: `src/ui/panel.rs:99` (`uninlined_format_args`) and `src/state/session.rs:997` (`needless_borrows_for_generic_args`). Root owns the small integration cleanup; ZOE-1 leaves unrelated files untouched.
- No release build, installation, model invocation, live mutation or push performed.

## Handoff

Implementation commit: `f32e0b8` (`feat(api): add incremental fleet console inventory`). Merged the latest integration tip `c44dc38` into the ticket branch as `b728161` (`chore(api): merge latest goal console integration`). Worktree is clean.

After the merge:

- `cargo test --locked --all-features`: all 226 tests passed (207 library, 9 binary, 10 public-console tests).
- `cargo fmt --all --check`: passed.
- `cargo check --lib --locked --no-default-features`: passed.
- `RUSTDOCFLAGS='-D warnings' cargo doc --locked --all-features --no-deps`: passed.

The console's `refresh` and first expansion perform blocking local IO. ZOE-4 must call these on its inventory worker/off the terminal UI thread; borrowing/rendering the cached graph performs no IO. Main TUI/default launch, runtime status, durable goal/question/artifact overlays and live controls remain the subsequent tickets. Search currently covers indexed project/session/agent names and compact prompts; generated-document-path search will need the ZOE-2 artifact records added to the graph's filter inputs. Claude projects fall back to the provider's opaque sanitized key where no explicit cwd metadata has been projected. No browser source changed; portable-core compilation passed, while the frontend-specific wasm check remains part of final integration verification.
