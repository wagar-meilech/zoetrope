# WG-2 implementation — exact work ownership

Implemented 27 October 2026 on `feat/work-owner`, starting from `dd7c744`.
Ticket: [WG-2](../work-graph-tickets/WG-2.md), published as issue #17.

## Public behavior

Reported collaboration paths remain in saved work and in the inspector's owner ID. The inventory worker joins those aliases to public provider metadata before selecting the inspection/control recipient. Codex's first `session_meta` supplies its native `payload.id`, exact root session, collaboration path and declared parent. Nested paths require a matching native parent chain in the same provider/root scope. Missing ancestry, invalid paths, internal review threads and ambiguous native matches cannot supply a recipient. Names, cwd, timestamps and newest-thread ordering do not participate.

`Console::owner_resolutions()` exposes the cached projection; it performs no file IO. The Fleet snapshot carries this projection, and its selected-agent route normalizes a reported alias once before requesting activity or native control. Known native IDs retain their exact identity, including historical/unloaded IDs whose live capability remains independently unavailable. Missing and ambiguous aliases show explicit guidance/queue/stop and feedback explanations, without coordinator fallback. Human/unassigned source owners can remain absent and receive the same non-steerable treatment; the WG-3 source projection remains separate from explicit report compatibility.

Opening feedback pins the resolved native `AgentKey`. Later owner reassignment and refreshed ambiguous metadata do not retarget that draft. Existing deliberate coordinator recovery, saved recipient identity and replay isolation remain intact. No private reasoning, KV, model connection, network operation or live project input was added.

Header discovery reads at most 4 MiB plus one byte and rejects oversized headers. The existing incremental metadata cache is reused. Ancestry traversal is bounded to 64 links, and drawing performs no metadata IO.

## Approved seams and TDD evidence

The TDD skill and its tests/mocking references were read before writing tests. The already-approved seams were used throughout: public Console/input/render with an independent Codex IPC peer, plus external fixture files and the executable. No tests call private parser/resolver helpers.

All commands below used `CARGO_TARGET_DIR=/tmp/zoetrope-wg-build/owner` and `--locked`; sandbox execution required the existing environment escalation workaround.

1. **Exact owner and pinned draft.** `cargo test --locked --test fleet_ui logical_work_owner_targets_its_native_thread_and_pins_an_open_draft -- --exact --nocapture`
   - RED: render showed `Recipient: Agent /root/parser`; the independent peer rejected the logical path as a native thread.
   - GREEN: alias normalization selected `child-thread`; the independent peer accepted exact child guidance, queue and interruption after work reassignment. The retained case also adds a conflicting alias while the feedback editor is open, forcing a metadata refresh without changing its recipient.
2. **Ambiguity is unavailable.** `cargo test --locked --test fleet_ui ambiguous_work_owner_disables_controls_without_rerouting_to_coordinator -- --exact --nocapture`
   - RED: availability merely requested a selection and did not explain the ambiguous owner.
   - GREEN: guidance, queue, stop and opening feedback explain `owner path is ambiguous`. The observe-only peer receives no native mutation.
3. **Provider/root/native ancestry.** `cargo test --locked --test console owner_paths_use_exact_root_and_native_ancestry_even_when_goals_are_collapsed -- --exact --nocapture`
   - RED: an out-of-scope parent made a valid nested path ambiguous, and missing ancestry was accepted.
   - GREEN: collapsed goals retain exact root mappings; wrong-root/missing parent chains are excluded, including a provider-qualified negative case.
4. **Bounded public headers.** `cargo test --locked --test console oversized_public_headers_do_not_supply_owner_identity -- --exact --nocapture`
   - RED: a header above 4 MiB supplied an owner mapping.
   - GREEN: oversized metadata is rejected; a normal 96 KiB instruction-bearing header still supplies its exact alias.

Complementary retained regressions cover a missing alias with a deceptively similar agent name, alias-based Commands/public narration inspection, existing unindexed native ownership, exact saved draft identity, control restrictions and read-only replay. These cases reuse behavior made green by the preceding slices.

## Checks

- `cargo test --locked --test console --test fleet_ui --test codex_control --test fleet_runtime`: pass (80 tests; one existing manual probe ignored).
- `cargo clippy --locked --all-targets -- -D warnings`: pass after applying the suggested `?` style simplification.
- `cargo test --locked`: pass, 328 tests; one existing manual resource probe ignored.
- `cargo check --locked --no-default-features`: pass.
- `CARGO_TARGET_DIR=/tmp/zoetrope-wg-build/owner/msrv cargo +1.88.0 check --locked`: pass.
- `cargo check --locked --no-default-features --target wasm32-unknown-unknown`: pass.
- `cargo fmt --all -- --check` and `git diff --check`: pass.
- Existing public large-history probe: `cargo test --locked --test fleet_ui large_fleet_placement_resource_probe -- --exact --ignored --nocapture`: pass. 1,000 histories loaded in 333 ms with 32,026 µs peak UI poll; 4,000 loaded in 818 ms with 80,493 µs peak UI poll. These observations describe this fixture/machine and impose no timing threshold; full record IO remains on the inventory worker.

Merged the latest integration tip `679f8b9` into this isolated branch with merge commit `80e2901`. That merge contains documentation only; the checked implementation is unchanged. `git merge-base --is-ancestor feat/goal-console HEAD` and a clean worktree verify the handoff baseline. No repeated broad checks were needed after the documentation merge.
