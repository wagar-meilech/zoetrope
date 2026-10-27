# Local installation implementation — 27 October 2026

Implements [INSTALLATION-SPEC](../INSTALLATION-SPEC.md), issue #14, through the
approved installer/updater/executable subprocess seam. Based on integration
`69e78b2`; current integration is merged before completion. No app CLI, Rust
runtime dependency, service, Git operation or implicit source update is added.

## Interface and behavior

- `./scripts/install.sh` defaults to `$HOME/.local/bin`.
- `./scripts/install.sh --bin-dir DIR` chooses an explicit destination;
  `--help` documents the option. Spaces and apostrophes in source/destination
  paths work.
- Build command: `cargo build --release --locked --bin zoe --target-dir DIR`,
  run from the physical checkout directory. The default target is its `target/`;
  `CARGO_TARGET_DIR` remains supported, with relative values based at the checkout.
- Both command artifacts are staged inside the destination after successful
  build, permissions set, then renamed. Existing command directories are rejected
  before either file changes. Build failures leave both installed commands intact.
- Installed `zoe` is the compiled artifact itself. Its normal launch invokes no
  Cargo or background service.
- Generated `zoe-update` records the physical checkout and destination using shell
  quoting, and invokes that checkout’s installer. No Git pull, branch change,
  reset, upstream crate installation or checkout deletion occurs. The updater
  fails with reinstall guidance if its source checkout has moved/disappeared.
- Only the README installation section changes. Root integrates and performs the
  actual connected-Linux installation from the persistent checkout, avoiding a
  temporary-worktree dependency. macOS portability is designed with POSIX shell
  and common utilities; execution evidence here is Linux only.

## Vertical TDD evidence

All tests are public processes against isolated fixtures. Cargo is replaced at
its external executable boundary with a fixture build tool which requires a
locked release `zoe` build and creates versioned executable candidates. No
application internals are mocked.

Each command uses `CARGO_TARGET_DIR=/tmp/zoetrope-sem-build/local-install`:

```sh
cargo test --locked --test local_install installing_to_the_user_bin_runs_the_release_without_cargo_at_launch -- --exact
```

RED: the installer process could not open the missing helper. GREEN: a default
install runs its installed command with an empty PATH, without needing Cargo.

```sh
cargo test --locked --test local_install updating_twice_from_another_directory_preserves_the_running_command -- --exact
```

RED: installation produced no updater. GREEN: two updater runs from another
project install the new version from the same checkout. Paths contain spaces and
apostrophes. A held old command completes with its original version, and an open
old artifact still reads the original bytes, verifying rename replacement rather
than truncating the live file.

```sh
cargo test --locked --test local_install a_moved_checkout_explains_reinstallation_without_breaking_the_installed_command -- --exact
```

RED: missing source produced only the shell’s exec error. GREEN: the updater
explains reinstallation, the installed app still runs, and reinstalling from the
moved checkout restores updates.

```sh
cargo test --locked --test local_install an_existing_command_directory_is_rejected_before_installed_files_change -- --exact
```

RED: the installer reported success after moving the executable into an existing
`zoe/` directory and replacing its sibling updater. GREEN: it rejects the invalid
target before replacing either installed artifact.

Additional public regression:
`a_failed_update_leaves_both_installed_commands_usable_and_unchanged` was already
green with the build-before-replace implementation; no artificial RED was
introduced. It forces Cargo exit27, verifies both old artifacts are unchanged,
runs the old command, then removes the failure and successfully updates.

## Acceptance checks

Run after all helper/test/document edits:

```sh
CARGO_TARGET_DIR=/tmp/zoetrope-sem-build/local-install cargo test --locked --test local_install
CARGO_TARGET_DIR=/tmp/zoetrope-sem-build/local-install cargo clippy --locked --all-targets --all-features -- -D warnings
cargo fmt --all --check
sh -n scripts/install.sh
./scripts/install.sh --help
git diff --check
```

All five installer/updater tests passed on the connected Linux environment.
Strict Rust 1.99 all-target/all-feature Clippy, formatting, shell syntax/help and
whitespace checks passed. The handoff reports the exact checked commit after
rechecking its tree. Full semantic/native regression validation is owned by
integration and need not repeat for shell-only installation work. Actual release-build/install evidence is recorded separately
by root on the persistent connected-Linux checkout.
