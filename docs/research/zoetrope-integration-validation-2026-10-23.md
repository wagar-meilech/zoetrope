# Integration validation

## ZOE5 real preview

- Packaged the integrated dependency-free bridge with `python3 editors/vscode/package_vsix.py --output .local/research-artifacts/tmp/zoetrope-preview-0.1.0.vsix`.
- Installed with `code --install-extension ... --force`: successful.
- Real installed VS Code 1.142.0 / MD Reader 0.0.8.
- Selected `/Users/melikhov/Repos/zoetrope/docs/GOAL-CONSOLE-SPEC.md`, dispatched its exact encoded `vscode://wagar.zoetrope-preview/open?uri=...` using macOS `open` outside the shell sandbox. Default sandbox dispatch fails LaunchServices; normal host dispatch succeeds.
- VS Code displayed its one-time extension-URI confirmation. Clicked Open without changing the remember preference.
- Computer-use AX verification showed selected tab `MD Reader - GOAL-CONSOLE-SPEC.md`, `AnandShah.mdreader` webview, heading `Zoetrope goal console` and actual rendered specification text. This is real render evidence, beyond unit host tests and dispatch acknowledgment.
- No editor renderer preference, security policy or unrelated user file changed.

## Integration checkpoints

- ZOE1+5 at `dfa63cf`: 231 tests passed, formatting and portable core passed via merger agent.
- Draft PR: https://github.com/wagar-meilech/zoetrope/pull/8

## Portable browser check

The root native crate remains on Rust1.88. The existing separately locked browser frontend requires Rust1.90 via beamterm1.0 dependencies. Installed Rust1.90 only for explicit browser checks, leaving the repository native override unchanged. `cargo +1.90.0 check --locked --offline --manifest-path web/wasm/Cargo.toml --target wasm32-unknown-unknown` passed after the fleet+preview integration. No dependency pins changed.

## 25 October preparation

Resumed from UI WIP3f5426f and integration70e6362. Official committed1.1.11 and typos1.51.1 macOS arm64 binaries remain available in .local/check-tools; SHA256 was verified against official GitHub release metadata when downloaded. No global installation. The root integration typo check passed after the documentation audit. Final repeat is needed when UI/source/docs change.

## 25 October pre-fix optimized candidate

Built integrationd853af2 optimized6,100,128byte executable. A build-path error briefly updated the installed target; immediately copied the candidate to target/zoe-candidate/zoe and atomically restored the original3,633,056byte installed executable from its existing Cargo artifact. Verified old --help is legacy and candidate --help is fleet. Final install remains pending review fixes.

Candidate ran in a120x40 owned PTY against real local history/current runtime, passive only. It displayed the actual task's4/6 criteria/evidence, H history and /goal search worked, c displayed real saved criteria, R entered explicitly read-only replay and q returned; secondq exited. No guidance/queue/stop/answer was sent. Before fixes, Current view CPU delta0.33sec over167.08sec was0.198% CPU, RSS8,256–8,832KiB; History sample9,216KiB. These are preliminary observations during ongoing native activity, not final post-fix metrics. Exact samples: zoetrope-candidate-resource-samples.json. No fixedfps fleet redraw observed; active replay intentionally renders while playing.

Independent final review produced8 actionable findings in zoetrope-final-review-2026-10-25.md. All assigned to the single zoe_review_fixes implementer in dedicated review-fixes worktree. Installed release and PR readiness remain pending.
