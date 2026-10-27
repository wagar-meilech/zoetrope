# Work graph integration evidence

## WG-2 — exact work ownership — 27 October 2026

- Integration branch: `feat/goal-console`.
- Previous integration tip: `679f8b9f1215786b095abc3a712c564bdc388fea`.
- Immutable source branch: `feat/work-owner`, checked clean at `3a7b69a3b61582e14accbc0d9de4246633c0dd4e` in `/tmp/zoetrope-wg2`.
- Source implementation commit: `f966cf55ccda2ef52d75b0c6dc4316de34cb3d7c`.
- Integration result: fast-forward to `3a7b69a3b61582e14accbc0d9de4246633c0dd4e`; no merge commit or conflicts. The source already included the preceding integration tip through `80e29019bc6333cea4edeb5cf35e5e574508b92d`.

The merger verified clean integration/source worktrees and the integration-tip ancestry before advancing the branch. Application and test content is identical to the checked implementation: `git diff --exit-code f966cf55ccda2ef52d75b0c6dc4316de34cb3d7c 3a7b69a3b61582e14accbc0d9de4246633c0dd4e -- src tests Cargo.toml Cargo.lock` passed. The merged root has the same trees/blobs:

| Path | Git object |
| --- | --- |
| `src` | `1ab0715b0c40db5b9fdc22db302faee244255e7a` |
| `tests` | `10d7dca5efc917dac1f090de58689003200aef99` |
| `Cargo.toml` | `a27ae937bd79b9960fdbd65bb2c143adf74ef2e6` |
| `Cargo.lock` | `be9833fc08bcd562f239033be23307d72dc1be38` |

The [WG-2 implementation evidence](wg-2-implementation.md) supplies the retained green results: targeted console/fleet UI/native-control/runtime suites (80 passed, one existing ignored probe), all 328 native tests, strict all-target Clippy, formatting/diff checks, portable core, Rust 1.88 and wasm target checks. Its public 1k/4k-history resource probe also passed. These checks used `CARGO_TARGET_DIR=/tmp/zoetrope-wg-build/owner`. They were not repeated solely to relocate the unchanged source through a fast-forward. The merger additionally checked `git diff --check` for the integration documentation and verified that these status/evidence updates leave the checked application/test trees unchanged.

WG-2 is marked `done` with `Integrated` in the canonical live task frontier. This records integration, not outcome verification or issue closure. Combined integration checks and independent Standards/Spec reviews remain pending under WG-6. WG-1/WG-3 were not merged by this handoff. No push, publication, installation, main-branch merge, live session input or project instruction change was performed.

## WG-1 — work/dependency canvas — 27 October 2026

- Previous integration tip: `05938ec113017384a0fb6dc512438999c006eda1`.
- Immutable source branch: `feat/work-canvas`, checked clean at `8d0e66a8f71394634f001b65a3982b4a352baeca` in `/tmp/zoetrope-wg1`.
- Integration result: fast-forward to `8d0e66a8f71394634f001b65a3982b4a352baeca`; no merge commit or conflicts. The checked source already contains the latest integration/WG-2 via `3319deb`.

The merger verified clean source/integration worktrees and ancestry from the previous integration tip. The fast-forward preserves the exact source tested in the [WG-1 implementation evidence](wg-1-implementation.md): **59 public fleet/control tests passed, one existing manual probe ignored**, with strict all-target Clippy, formatting and diff checks green. These checks used `CARGO_TARGET_DIR=/tmp/zoetrope-wg-build/canvas`. No broad checks were repeated for this unchanged application/test tree. The merged `src` tree is `e01aa08953be66b44aae59895028ef023f17d422`, the `tests` tree is `3500c71ac79a576ad85a9dca5935094e8b8b274d`, and Cargo manifest/lock blobs remain `a27ae937bd79b9960fdbd65bb2c143adf74ef2e6` / `be9833fc08bcd562f239033be23307d72dc1be38`. Documentation-only frontier/evidence updates passed the staged diff check and leave these application/test trees unchanged.

WG-1's immutable ancestry also contains the parent's explicitly authorized green WG-3 interfaces `c1c3f07` and `55c0115`. This establishes only that earlier source/import interface state. WG-3 remains **active** while its source hardening and final validation continue; no uncommitted WG-3 changes were merged and its ticket is not complete.

The canonical frontier marks WG-1 `done` / `Integrated` and WG-5 `active` with owner `/root/wg5_visual`, making visual legibility work ready to start from this integration. Combined checks and independent final Standards/Spec reviews remain pending under WG-6. No push, publication, installation, main-branch merge or live/native input was performed.

## WG-3 — structured local project sources — 27 October 2026

- Previous integration tip: `55d35bb98a3942955d2d4a00e1ad25f7743b0ce0`.
- Immutable source branch: `feat/work-sources`, checked clean at `b9afbb94b77f92b025ef04a45c2eecc3e58542eb` in `/tmp/zoetrope-wg3`.
- Integration result: fast-forward to `b9afbb94b77f92b025ef04a45c2eecc3e58542eb`; no merge commit or conflicts. Verified ancestry includes the previous integration tip and WG-2's native identity-proof follow-up `d1d6344dbf5ccbc0d365e26dfc65a8251e8d4bd2`.

The final source-only owner guard is preserved exactly: direct source-native IDs require scoped public metadata proof; missing/foreign/malformed identities cannot provide a steering recipient, ancestry-proven identities recover their exact native recipient, and exact native root identity normalizes to the coordinator. Raw source owner attribution and legacy explicit-report compatibility remain intact.

The fast-forward retains the exact checked source: `src` tree `45c3967824302a07a7e8933c2dbc9e8dc5e0b621`, `tests` tree `8d3935b0810f3eb89801a60a03b805bf3f34057a`; Cargo manifest/lock blobs remain `a27ae937bd79b9960fdbd65bb2c143adf74ef2e6` / `be9833fc08bcd562f239033be23307d72dc1be38`. The [WG-3 implementation evidence](wg-3-implementation.md) records **16 tracking tests and five targeted source UI tests passed, one existing source UI probe ignored**, plus strict all-target Clippy, formatting and diff checks on the final source. Earlier combined native, portable/no-default-feature and Rust 1.88 checks passed before the final source-owner guard; they are retained as qualified earlier evidence, with the full final combined root run still pending. No checks were repeated merely to relocate this identical tree. Documentation-only integration updates passed the staged diff check and leave application/test content unchanged.

The [source contract](wg-3-source-contract.md) is available for the tracking-skill consumer. The canonical frontier marks WG-3 `done` / `Integrated` and WG-4 `active`, owner `/root/wg4_skill`. This records ticket integration rather than outcome verification, issue closure or final independent review. WG-6 still owns final combined checks and independent Standards/Spec reviews. No push, publication, installation, main-branch merge, native input or other-project changes were performed.

## WG-4 — reusable tracking skill — 27 October 2026

- Previous integration tip: `ee34502e3d9d12c7601e2a4afdd4b4c16247606a`.
- Immutable source branch: `feat/tracking-skill`, checked clean at `fa55b5783e46a5ebb9d7a33e39ab7f7facdbb455` in `/tmp/zoetrope-wg4`.
- Integration result: fast-forward to `fa55b5783e46a5ebb9d7a33e39ab7f7facdbb455`; no merge commit or conflicts. Verified source ancestry already includes the previous integration tip and its scoped SourceTask owner guards/work graph.

The merger verified clean source/integration worktrees, the exact source hash and integration-tip ancestry before advancing the branch. WG-4's Rust changes only extend the exact-scope tracking activation draft; source-native ownership proof and graph behavior remain unchanged. The merged application, test, installer and skill content is identical to the checked source. Retained Git objects:

| Path | Git object |
| --- | --- |
| `src` | `77a009ee439c238694600200a918cdb7e632c310` |
| `tests` | `d3a60fb2fd391de2340c42a3be19097722979fec` |
| `scripts` | `1f2a1840d75ada617c70e0a417b458994ea30009` |
| `skills` | `0426e642cba56f3dc7ed5294a1cbbea0cbb95cde` |
| `Cargo.toml` | `a27ae937bd79b9960fdbd65bb2c143adf74ef2e6` |
| `Cargo.lock` | `be9833fc08bcd562f239033be23307d72dc1be38` |

The [WG-4 implementation evidence](wg-4-implementation.md) supplies retained green results: **60 public fleet UI tests passed, one existing resource probe ignored; 16 tracking tests and all seven installer tests passed**, plus strict all-target Clippy, formatting, shell syntax, diff checks and the skill validator. These checks used `CARGO_TARGET_DIR=/tmp/zoetrope-wg-build/skill`; installer fixtures isolated both `HOME` and `CODEX_HOME`. The independent disposable-project skill exercises passed Matt-style setup, exact new-goal binding/resume, disable, opted-out continuation with an old committed pointer, and a non-Matt fallback with two real tasks. This evidence is limited to those scenarios and the recorded check scope. No checks were repeated merely to relocate this unchanged source through a fast-forward. Documentation-only frontier/evidence updates passed the staged diff check and leave the checked application/test/installer/skill content unchanged.

The canonical frontier marks WG-4 `done` / `Integrated`. WG-5 remains active and must include this integration before its handoff. WG-6 still owns combined final checks, independent Standards/Spec reviews, HANDOFF updates and the live installation; integration does not establish outcome verification or issue closure. No push, publication, global installation, main-branch merge, native input or other-project changes were performed.

## WG-5 — visual work graph and controls — 27 October 2026

- Previous integration tip: `c4b3855a1aa20650fe2b40ec4964d68b014cb48f`.
- Immutable source branch: `feat/work-visuals`, checked clean at `4681c772fc2294a3fd2efaf2c1eff1d5bce188d8` in `/tmp/zoetrope-wg5`.
- Integration result: fast-forward to `4681c772fc2294a3fd2efaf2c1eff1d5bce188d8`; no merge commit or conflicts. Verified ancestry includes the previous integration tip through `5887980`, preserving WG-3's scoped source-owner safeguards and WG-4's exact tracking activation.

The merger verified clean source/integration worktrees, the exact source hash and integration-tip ancestry before advancing the branch. The final source's application/test/package content matches its checked topic integration `5887980`; the final topic commit only adds documentation and archived terminal evidence. The merged root retains these checked Git objects:

| Path | Git object |
| --- | --- |
| `src` | `54131f00d3b1d9c0c56d86e09f15624ac4983bd4` |
| `tests` | `9437c5162011f14f71ce113c4dc62056abf16f41` |
| `scripts` | `1f2a1840d75ada617c70e0a417b458994ea30009` |
| `skills` | `0426e642cba56f3dc7ed5294a1cbbea0cbb95cde` |
| `Cargo.toml` | `a27ae937bd79b9960fdbd65bb2c143adf74ef2e6` |
| `Cargo.lock` | `be9833fc08bcd562f239033be23307d72dc1be38` |

The [WG-5 implementation evidence](wg-5-implementation.md) supplies retained final green results: **69 public UI tests passed, one existing manual resource probe ignored; eight real-terminal/runtime tests and eight executable tests passed**, plus strict all-target Clippy, formatting and diff checks. These checks used `CARGO_TARGET_DIR=/tmp/zoetrope-wg-build/visuals` and include source-owner guards, activation drafts, recipient pinning, navigation, graph limits and adaptive overview regressions. The [six final terminal captures](wg-5-terminal-captures/README.md) archive PNG, plain-cell and actual ANSI evidence for readable/overview views at 150×42, 106×48 and 60×28, using disposable synthetic state. The visual agent inspected all six and root inspected the improved 106×48 adaptive overview. This establishes those fixtures and recorded check scope, not every terminal or live-project behavior. No checks were repeated merely to relocate the identical source. Documentation-only frontier/evidence updates passed the staged diff check and preserve the checked application/test/installer/skill content.

The canonical frontier marks WG-5 `done` / `Integrated` and WG-6 `active`, owner `main`. All implementation tickets are now integrated; root/WG-6 owns combined native/portable/browser/MSRV checks, independent Standards/Spec reviews, final handoff and live installation. Integration does not establish outcome verification or issue closure. No push, publication, global installation, main-branch merge, native input or other-project changes were performed.

## WG-6 — final review fixes — 27 October 2026

- Previous integration tip: `bfbadce1ad3784e2d946930ddb49a7d929c5671d`.
- Immutable source branch: `feat/work-graph-review-fixes`, checked clean at `2a65e3155310768a34833a4f0b762c4e6afc72df` in `/tmp/zoetrope-wg-fixes`.
- Integration result: fast-forward to `2a65e3155310768a34833a4f0b762c4e6afc72df`; no merge commit or conflicts. The checked source already contains the previous integration tip. Its final topic merge leaves the application/test/package/browser inputs identical to validated topic commit `2fd3ac5`.

The merger verified clean source/integration worktrees, exact source identity and integration-tip ancestry. The fixes reject nonregular sources before opening and use nonblocking Unix open plus descriptor validation; duplicate IDs across selected ticket files retain stale last-good state; visual cues use typed statuses; auto color preference passes both supported compiler lints; contention fixtures lock the exact public goal record. Scoped source-owner guards, exact activation and native recipient pinning remain in the integrated ancestry. Retained checked objects:

| Path | Git object |
| --- | --- |
| `src` | `d88231676f16e4d23f5c024bcd4838f65eebbcb9` |
| `tests` | `ae6fa9e97272d9ce2a28585520a33de20c160379` |
| `scripts` | `1f2a1840d75ada617c70e0a417b458994ea30009` |
| `skills` | `0426e642cba56f3dc7ed5294a1cbbea0cbb95cde` |
| `Cargo.toml` | `244094e91689a84305dd54acbfee8b69b194c222` |
| `Cargo.lock` | `2b643b079db6fb4cd004f1c35a5e61d539089968` |

The [review-fix evidence](work-graph-review-fixes.md) supplies retained green results on Rust 1.88 and 1.99: **18 tracking, 69 UI and eight PTY/runtime tests passed**, with one existing manual UI resource probe ignored; normal parallel UI suites and strict all-target Clippy also passed on both compilers after the contention-fixture correction. Portable checks passed 176 library and 13 Console tests; the excluded WebAssembly workspace check, formatting and diff checks passed. The evidence records both public RED/GREEN regressions and qualifies the earlier nondeterministic contention failure. No tests were repeated merely to relocate identical checked inputs. Documentation-only integration updates passed the staged diff check and leave the checked implementation/test/package/browser content unchanged.

WG-6 remains `active`, owner `main`. Root owns final full-suite, package/rustdoc, resources, installation, live observation and independent reviewer rechecks; earlier baseline results are not final-source verification. No push, publication, global installation, main-branch merge, native input or other-project changes were performed.
