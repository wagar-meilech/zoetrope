# WG-4 tracking skill implementation

27 October 2026. Isolated `feat/tracking-skill`, based on integration `ee34502`.
No live native input, global installation, external project edits or publication.

## Delivered behavior

- Packaged normally discoverable `skills/zoetrope-tracking/SKILL.md`, with setup,
  structured-source and optional-report references. The main skill keeps workers
  on ordinary assigned task artifacts; coordinator setup details are disclosed.
- Setup uses the project's existing instruction-to-tracker convention, preserving
  upstream Matt skills and task prose. Only missing source fields and bounded
  managed instruction blocks are added. Non-Matt setup reuses an adequate plan or
  seeds one minimal document with known work; an empty starter cannot silently
  replace useful native/report work.
- Project opt-in is separate from exact native goal/source scope. Coordinator
  continuation checks local state, validates current runtime identity, and leaves
  an opted-out project disabled even when old committed guidance reappears.
- Enable tracking still prepares a saved editable coordinator draft. It names the
  installed skill path, executable, exact provider/root, project and state root,
  authorizes narrow setup only when sent, and retains compact reporting fallback.
  Existing drafts, explicit sending and exact-owner guards are preserved.
- `scripts/install.sh` and its generated `zoe-update` install the skill into
  `${CODEX_HOME:-$HOME/.codex}/skills/zoetrope-tracking` after a successful locked
  build. The updater remembers that destination. Fresh packaged files are staged
  before preserving user additions, so old file symlinks cannot redirect writes.
  Unrelated skills/extras and the previous app/skill after failed builds survive.

## Public command interface

No new tracking verbs or schema were introduced; WG-3 remains the executable
contract. The UI emits a runnable exact-scope status command and a compact context
command. Setup supplies the selected source flags:

```sh
'/installed/zoe' tracking status --project '/project' --provider codex --session 'EXACT_ROOT' --root '/local/state'
'/installed/zoe' tracking enable --project '/project' --provider codex --session 'EXACT_ROOT' --source 'docs/WORK.md#Tasks' --root '/local/state'
'/installed/zoe' tracking disable --project '/project' --provider codex --session 'EXACT_ROOT' --root '/local/state'
'/installed/zoe' tracking sync --project '/project' --provider codex --session 'EXACT_ROOT' --root '/local/state'
'/installed/zoe' state --root '/local/state' context codex 'EXACT_ROOT'
```

`--source` repeats for explicit ticket files (`tickets/T1.md@ticket`) or tables.
Codex tracking commands also accept `--session auto`, validating both current
runtime identity variables and public ancestry. `state context` needs the resolved
exact root, not `auto`. Ordinary source edits use the existing background importer;
`sync` is optional recovery. Source completion never verifies outcomes.

## RED / GREEN slices

All commands used `CARGO_TARGET_DIR=/tmp/zoetrope-wg-build/skill`; all installer
fixtures set both isolated `HOME` and isolated `CODEX_HOME` before any installation.
The approved seams were console input/render plus public saved-state/executable
commands, and installer/external processes/files. No private-helper tests or wording
snapshots were added.

1. `cargo test --locked --test local_install installation_and_updates_ship_the_skill_without_replacing_unrelated_content -- --exact`
   - RED: installed `SKILL.md` was missing.
   - GREEN: packaged skill and reference installed/updated, personal extra and
     neighboring skill survived, failed build retained previous skill. Entire
     installer suite then passed (six tests).
2. `cargo test --locked --test fleet_ui work_focused_coordinator_and_tracking_actions_stay_explicit_and_request_missing_criteria -- --exact`
   - RED: old draft lacked the installed skill/exact tracking command.
   - GREEN: saved editable draft names installed skill/executable; executing its
     quoted status command returns the selected project/provider/root and disabled
     opt-in without enabling it. Child draft, criteria request and empty outbox
     assertions remain. Existing feedback-preservation test retains its checks;
     an obsolete prose-only “meaningful transitions” assertion was removed.
3. `cargo test --locked --test local_install updating_a_packaged_file_replaces_a_user_symlink_without_overwriting_its_target -- --exact`
   - RED: overlay followed old installed `SKILL.md` symlink and replaced unrelated
     target content.
   - GREEN: new package staged first, user extras copied without replacing it;
     installed skill is regular packaged content and original target is unchanged.
     Entire installer suite then passed (seven tests).

## Independent disposable behavioral exercise

The skill-creator-authorized evaluator received only the skill path, realistic
requests, isolated executable/state/identity configuration and raw project files.
It did not read implementation notes or parent discussion. All writes stayed in
`/tmp/zoetrope-wg4-behavior`; synthetic public session headers represented
`matt-goal`, `matt-next` and `plain-goal`. Commands used the actual debug executable
and `--session auto`; no native messages were sent.

1. **Matt-style setup:** `CLAUDE.md` already pointed to a project tracker; two
   individual tickets had titles/status/owners/dependencies and acceptance prose,
   but no explicit metadata IDs. Evaluator added just those IDs and one managed
   block in the tracker. Both explicit `@ticket` sources imported one task each,
   with no stale/error flags; context showed T1 active, T2 planned and T2 → T1.
   Existing instruction routing, status/owner text and ticket prose survived.
2. **New goal/resume:** a new exact native root `matt-next` continued the work and
   updated T1's ordinary status to reported done. Its own selected source binding
   imported successfully; original `matt-goal` binding status was byte-for-byte
   unchanged at that point (shared source edits can appear on its later refresh).
   No instruction rewrite, per-task JSON apply or outcome verification occurred.
3. **Disable:** explicit disable stopped both project bindings and removed only
   the managed tracker block. Original instructions, both tickets, task history
   and source snapshots survived.
4. **Old pointer continuation:** harness restored the prior committed tracker
   block, simulating a checkout with old guidance. A normal resume changed only
   T2's ordinary artifact to in-progress; status remained `project_opt_in: false`
   and `enabled: false`, and cached source snapshots did not refresh.
5. **No-Matt setup:** raw README described adding a pause button then checking
   pause/resume. Evaluator created one `docs/WORK.md#Tasks` plan containing those
   two planned tasks and the declared prerequisite, a managed project tracker
   block and tiny `AGENTS.md` pointer. README was preserved. `plain-goal` imported
   two tasks with no stale/error flags; no empty plan or parallel report was bound.

The implementing agent inspected the resulting files and reran public status
commands: Matt remained disabled; plain project had exactly its selected table,
with two tasks and no errors. No evaluator usability trouble was reported. This
establishes these scenarios, not universal arbitrary-Markdown or model compliance.

## Final WG-4 checks

- `cargo test --locked --test fleet_ui --test local_install --test tracking`:
  console 60 passed / one intentionally ignored resource probe; source 16 passed;
  installer six passed at that point, then all seven passed after the symlink fix.
- `cargo clippy --locked --all-targets -- -D warnings`: passed after final changes.
- `cargo fmt -- --check`, `git diff --check`, `sh -n scripts/install.sh`: passed.
- `python3 /home/dust_60/.codex/skills/.system/skill-creator/scripts/quick_validate.py skills/zoetrope-tracking`:
  `Skill is valid!`.

Combined integration checks, live installation and HANDOFF updates belong to
WG-6/root. No global skill or app was installed by WG-4.
